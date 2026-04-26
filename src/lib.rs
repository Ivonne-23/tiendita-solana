use anchor_lang::prelude::*;

declare_id!("Dkg7j4e1zkho7zcZdKnX6K15Sjsi2K3KcV8N9VPfEK5r");

#[program]
pub mod inventario_tienda {
    use super::*;


    pub fn inicializar_tienda(ctx: Context<InicializarTienda>, nombre: String) -> Result<()> {
        let tienda = &mut ctx.accounts.tienda_db;
        tienda.nombre = nombre;
        tienda.admin = *ctx.accounts.admin.key;
        tienda.total_productos = 0; // Se inicializa el contador de inventario
        Ok(())
    }

    /// @notice Registra un nuevo producto vinculándolo al administrador.
    /// @dev La PDA del producto usa el 'codigo' como semilla para asegurar unicidad.
    /// @param precio Valor en lamports
    /// @param stock Cantidad inicial de artículos disponibles.
    pub fn agregar_producto(
        ctx: Context<AgregarProducto>,
        _codigo: String,
        nombre: String,
        precio: u64,
        stock: u64,
        categoria: String,
    ) -> Result<()> {
        let producto = &mut ctx.accounts.producto;
        producto.nombre = nombre;
        producto.precio = precio;
        producto.stock = stock;
        producto.categoria = categoria;
        producto.admin = *ctx.accounts.admin.key;

        // Incrementa el contador global en la cuenta maestra de la tienda
        let tienda = &mut ctx.accounts.tienda_db;
        tienda.total_productos += 1;

        msg!("Evento: Producto '{}' registrado exitosamente", nombre);
        Ok(())
    }

    /// @notice Modifica el stock disponible de un producto específico.
    /// @dev Valida automáticamente que el administrador que firma sea el dueño del producto.
    pub fn actualizar_stock(
        ctx: Context<ActualizarProducto>,
        _codigo: String,
        nuevo_stock: u64,
    ) -> Result<()> {
        let producto = &mut ctx.accounts.producto;
        producto.stock = nuevo_stock;
        msg!("Evento: Stock actualizado a {}", nuevo_stock);
        Ok(())
    }

    /// @notice Elimina físicamente el registro del producto de la red.
    /// @dev Cierra la cuenta y devuelve los SOL depositados por renta al administrador.
    pub fn eliminar_producto(_ctx: Context<EliminarProducto>, codigo: String) -> Result<()> {
        msg!("Evento: Cuenta cerrada. Código de producto eliminado: {}", codigo);
        Ok(())
    }
}

//ESTRUCTURAS DE DATOS (ESTADO) 

#[account]
#[derive(InitSpace)]
/// Representa la base de datos de la tienda.
pub struct TiendaDB {
    pub admin: Pubkey,        
    #[max_len(40)]            // Soporta nombres de tienda de hasta 40 caracteres
    pub nombre: String,
    pub total_productos: u64, 
}

#[account]
#[derive(InitSpace)]
/// Estructura que define un producto individual dentro del inventario.
pub struct Producto {
    pub admin: Pubkey,
    #[max_len(30)]            // Nombre del producto
    pub nombre: String,
    pub precio: u64,
    pub stock: u64,
    #[max_len(20)]            // Categoría 
    pub categoria: String,
}

// CONTEXTOS (VALIDACIÓN Y ACCESO) 

#[derive(Accounts)]
#[instruction(nombre: String)]
pub struct InicializarTienda<'info> {
    /// Crea la cuenta Maestra. Semillas: ["tienda", admin_pubkey]
    #[account(
        init,
        payer = admin,
        space = 8 + TiendaDB::INIT_SPACE, 
        seeds = [b"tienda", admin.key().as_ref()],
        bump
    )]
    pub tienda_db: Account<'info, TiendaDB>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(codigo: String)]
pub struct AgregarProducto<'info> {
    /// Referencia a la cuenta de la tienda para actualizar estadísticas
    #[account(mut, seeds = [b"tienda", admin.key().as_ref()], bump)]
    pub tienda_db: Account<'info, TiendaDB>,

    /// Crea la PDA del Producto. Semillas: ["producto", admin_pubkey, codigo]
    #[account(
        init,
        payer = admin,
        space = 8 + Producto::INIT_SPACE,
        seeds = [b"producto", admin.key().as_ref(), codigo.as_bytes()],
        bump
    )]
    pub producto: Account<'info, Producto>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(codigo: String)]
pub struct ActualizarProducto<'info> {
    /// Localiza la PDA y verifica que el firmante sea el 'admin' guardado en la cuenta
    #[account(
        mut,
        seeds = [b"producto", admin.key().as_ref(), codigo.as_bytes()],
        bump,
        has_one = admin @ ErrorCode::Unauthorized 
    )]
    pub producto: Account<'info, Producto>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(codigo: String)]
pub struct EliminarProducto<'info> {
    /// Cierra la cuenta y transfiere el saldo de renta (lamports) al administrador
    #[account(
        mut,
        seeds = [b"producto", admin.key().as_ref(), codigo.as_bytes()],
        bump,
        has_one = admin,
        close = admin 
    )]
    pub producto: Account<'info, Producto>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[error_code]
/// Errores personalizados para la experiencia del usuario
pub enum ErrorCode {
    #[msg("No tienes autorización para modificar este producto.")]
    Unauthorized,
}
