use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("9Cmf94avwuwUo5zt8KphWJ68EoNjiCphrusdzMQE7Boi");

#[program]
pub mod simplified_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let bump = ctx.bumps.program_state;
        ctx.accounts.program_state.admin = admin;
        ctx.accounts.program_state.total_deposited = 0;
        ctx.accounts.program_state.bump = bump;
        Ok(())
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        // 将 SOL 从用户转移到 vault
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? += amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? -= amount;
        
        // 记录存款
        ctx.accounts.vault_state.total_deposited = ctx.accounts.vault_state
            .total_deposited
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
            
        Ok(())
    }

    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        // 验证 vault 有足够的余额
        let vault_lamports = ctx.accounts.vault.to_account_info().lamports();
        require!(
            vault_lamports >= amount,
            ErrorCode::InsufficientBalance
        );
        
        // 从 vault 转移 SOL 到用户
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;
        
        // 更新总存款
        ctx.accounts.program_state.total_deposited = ctx.accounts.program_state
            .total_deposited
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;
        
        Ok(())
    }

    pub fn withdraw_token(ctx: Context<WithdrawToken>, amount: u64) -> Result<()> {
        // 获取 bump 用于 PDA 签名
        let bump = ctx.bumps.program_state;
        
        // 验证 vault 有足够的代币余额
        require!(
            ctx.accounts.vault_token_account.amount >= amount,
            ErrorCode::InsufficientBalance
        );
        
        // 构建 PDA 签名 seeds
        let bump_array = [bump];
        let seeds = &[
            b"program_state".as_ref(),
            &bump_array[..],
        ];
        let signer = &[&seeds[..]];
        
        // 从 vault 转移代币到用户
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.program_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;
        
        Ok(())
    }

    pub fn admin_withdraw_sol(ctx: Context<AdminWithdrawSol>, amount: u64) -> Result<()> {
        // 验证调用者是管理员
        require!(
            ctx.accounts.admin.key() == ctx.accounts.program_state.admin,
            ErrorCode::Unauthorized
        );
        
        // 验证 vault 有足够的余额
        let vault_lamports = ctx.accounts.vault.to_account_info().lamports();
        require!(
            vault_lamports >= amount,
            ErrorCode::InsufficientBalance
        );
        
        // 从 vault 转移 SOL 到管理员
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? += amount;
        
        Ok(())
    }

    pub fn admin_withdraw_token(ctx: Context<AdminWithdrawToken>, amount: u64) -> Result<()> {
        // 验证调用者是管理员
        require!(
            ctx.accounts.admin.key() == ctx.accounts.program_state.admin,
            ErrorCode::Unauthorized
        );
        
        // 验证 vault 有足够的代币余额
        require!(
            ctx.accounts.vault_token_account.amount >= amount,
            ErrorCode::InsufficientBalance
        );
        
        // 获取 bump 用于 PDA 签名
        let bump = ctx.bumps.program_state;
        
        // 构建 PDA 签名 seeds
        let bump_array = [bump];
        let seeds = &[
            b"program_state".as_ref(),
            &bump_array[..],
        ];
        let signer = &[&seeds[..]];
        
        // 从 vault 转移代币到管理员
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.admin_token_account.to_account_info(),
            authority: ctx.accounts.program_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;
        
        Ok(())
    }
}

// ========== 账户结构体 ==========

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        space = 8 + ProgramState::LEN,
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    /// CHECK: 这是 vault 的 SOL 账户
    pub vault: SystemAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"program_state"],
        bump
    )]
    pub vault_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    /// CHECK: 这是 vault 的 SOL 账户
    pub vault: SystemAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AdminWithdrawSol<'info> {
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    /// CHECK: 这是 vault 的 SOL 账户
    pub vault: SystemAccount<'info>,
    
    #[account(
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminWithdrawToken<'info> {
    pub admin: Signer<'info>,
    
    #[account(
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub admin_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

// ========== 状态账户 ==========

#[account]
pub struct ProgramState {
    pub admin: Pubkey,
    pub total_deposited: u64,
    pub bump: u8,
}

impl ProgramState {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin: Pubkey
        8 +  // total_deposited: u64
        1;   // bump: u8
}

// ========== 错误代码 ==========

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Underflow occurred")]
    Underflow,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Unauthorized")]
    Unauthorized,
}

