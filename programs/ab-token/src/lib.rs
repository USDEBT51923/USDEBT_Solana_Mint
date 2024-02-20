use anchor_lang::prelude::*;
use anchor_spl::{
    token::{
        self,
        Token,
        TokenAccount,
        Transfer,
        MintTo
    }
};

pub mod account;
pub mod constants;
pub mod error;

use error::*;
use account::*;
use constants::*;

declare_id!("4gLzvquJn1FJJ3hGFHwWpadBSpBxWk8x7GqmvLJshiE4");

#[program]
pub mod ab_token {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        _global_bump: u8,
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        global_authority.admin = ctx.accounts.admin.key();
        global_authority.old_token = OLD_TOKEN.parse::<Pubkey>().unwrap();
        global_authority.new_token = NEW_TOKEN.parse::<Pubkey>().unwrap();
        msg!("SuperAdmin: {:?}", global_authority.admin.key());
        Ok(())
    }

    pub fn update_old_token(
        ctx: Context<UpdateGlobalState>,
        _global_bump: u8,
        old_token: Option<Pubkey>
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
 
        require!(ctx.accounts.admin.key() == global_authority.admin.key(), ABTokenError::InvalidSuperOwner);

        if let Some(updated_old_token) = old_token {
            msg!("Old Token Changed to {:?}", updated_old_token);
            global_authority.old_token = updated_old_token;
        }
        Ok(())
    }

    pub fn update_new_token(
        ctx: Context<UpdateGlobalState>,
        _global_bump: u8,
        new_token: Option<Pubkey>
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
 
        require!(ctx.accounts.admin.key() == global_authority.admin.key(), ABTokenError::InvalidSuperOwner);

        if let Some(update_new_token) = new_token {
            msg!("New Token Changed to {:?}", update_new_token);
            global_authority.new_token = update_new_token;
        }
        Ok(())
    }

    pub fn token_transfer_mint_to<'info>(
        ctx: Context<TokenMintTo>,
        amount: u64
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        let from = ctx.accounts.old_token.to_account_info().clone();
        let authority = ctx.accounts.user.to_account_info().clone();
        let to = ctx.accounts.token_treasury.to_account_info().clone();
 
        let cpi_ctx: CpiContext<_> = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from,
                authority,
                to
            },
        );
        token::transfer(cpi_ctx, amount)?;
    
        let cpi_accounts = MintTo {
            mint: ctx.accounts.new_token.to_account_info().clone(),
            to: ctx.accounts.user.to_account_info().clone(),
            authority: global_authority.to_account_info().clone(),
        };

        token::mint_to(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount
        )?;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 8 + 32 + 32 + 32, 
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
        payer = admin
    )]
    pub global_authority: Box<Account<'info, GlobalInfo>>,

    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct UpdateGlobalState<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Box<Account<'info, GlobalInfo>>,

    #[account(mut)]
    pub admin: Signer<'info>,
    
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct TokenMintTo<'info> {

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Box<Account<'info, GlobalInfo>>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub old_token: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub new_token: Box<Account<'info, TokenAccount>>,

    /// CHECK instruction will fail if wrong edition is supplied
    #[account(
        mut,
        constraint = token_treasury.key() == TOKEN_TREASURY.parse::<Pubkey>().unwrap()
    )]
    pub token_treasury: AccountInfo<'info>,

    token_program: Program<'info, Token>

}