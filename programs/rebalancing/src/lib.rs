use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
mod amm_instruction;
use amm_instruction::{ SwapInstructionBaseIn, AmmInstruction};
use anchor_lang::solana_program;
// use amm_anchor::accounts::ProxySwapBaseIn;
// use amm_anchor::instructions::proxy_swap_base_in::swap_base_in;
// use amm_anchor::instructions::swap_base_in;
// use amm_anchor::SwapBaseIn;

declare_id!("2RNkDWBs3fCeA6r6aAA9fBfAx6A4fgQ8V24mutQkZyEt");

#[program]
pub mod rebalancing {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.rebalancing_active = false;
        Ok(())
    }

    pub fn rebalance<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Rebalance<'info>>,
        token_mints: Vec<Pubkey>,
        target_weights: Vec<u8>,
    ) -> Result<()> {
        // Validate inputs
        require!(
            token_mints.len() == target_weights.len(),
            RebalancingError::InvalidInputLength
        );

        // Verify weights sum to 100
        let weight_sum: u8 = target_weights.iter().sum();
        require!(weight_sum == 100, RebalancingError::InvalidWeights);

        // Get current token balances and calculate total value
        let mut total_value: u64 = 0;
        let mut current_balances: Vec<u64> = vec![];
        
        for i in 0..token_mints.len() {
            let vault = &ctx.remaining_accounts[i];
            let amount = Account::<TokenAccount>::try_from(vault)?.amount;
            current_balances.push(amount);
            total_value += amount; // Assuming all tokens have same decimals for simplicity
        }

        // Calculate current weights as percentages
        let mut current_weights: Vec<u8> = current_balances
            .iter()
            .map(|balance| ((balance * 100) as u128 / total_value as u128) as u8)
            .collect();

        // For each token that needs rebalancing
        for i in 0..token_mints.len() {
            if current_weights[i] > target_weights[i] {
                // Need to sell this token
                let amount_to_sell = calculate_swap_amount(
                    current_balances[i],
                    current_weights[i],
                    target_weights[i]
                );

                // Execute Raydium swap
                let cpi_program = ctx.accounts.amm_program.to_account_info();
                let cpi_accounts = SwapBaseIn {
                    amm: ctx.accounts.amm.to_account_info(),
                    amm_authority: ctx.accounts.amm_authority.to_account_info(),
                    amm_open_orders: ctx.accounts.amm_open_orders.to_account_info(),
                    amm_coin_vault: ctx.accounts.amm_coin_vault.to_account_info(),
                    amm_pc_vault: ctx.accounts.amm_pc_vault.to_account_info(),
                    market_program: ctx.accounts.market_program.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    market_bids: ctx.accounts.market_bids.to_account_info(),
                    market_asks: ctx.accounts.market_asks.to_account_info(),
                    market_event_queue: ctx.accounts.market_event_queue.to_account_info(),
                    market_coin_vault: ctx.accounts.market_coin_vault.to_account_info(),
                    market_pc_vault: ctx.accounts.market_pc_vault.to_account_info(),
                    market_vault_signer: ctx.accounts.market_vault_signer.to_account_info(),
                    user_token_source: ctx.accounts.user_token_source.to_account_info(),
                    user_token_destination: ctx.accounts.user_token_destination.to_account_info(),
                    user_source_owner: ctx.accounts.user_source_owner.clone(),
                    token_program: ctx.accounts.token_program.clone(),
                    amm_program: ctx.accounts.amm_program.to_account_info(),
                };
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                swap_base_in(cpi_ctx, amount_to_sell, 0)?;
                // amm_anchor::instructions::swap_base_in(cpi_ctx, amount_to_sell, 0)?;
            }
        }

        Ok(())
    }
}

pub fn swap_base_in<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, SwapBaseIn<'info>>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    let ix = amm_instruction::swap_base_in(
        ctx.program.key,
        ctx.accounts.amm.key,
        ctx.accounts.amm_authority.key,
        ctx.accounts.amm_open_orders.key,
        ctx.accounts.amm_coin_vault.key,
        ctx.accounts.amm_pc_vault.key,
        ctx.accounts.market_program.key,
        ctx.accounts.market.key,
        ctx.accounts.market_bids.key,
        ctx.accounts.market_asks.key,
        ctx.accounts.market_event_queue.key,
        ctx.accounts.market_coin_vault.key,
        ctx.accounts.market_pc_vault.key,
        ctx.accounts.market_vault_signer.key,
        ctx.accounts.user_token_source.key,
        ctx.accounts.user_token_destination.key,
        ctx.accounts.user_source_owner.key,
        amount_in,
        minimum_amount_out,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

#[account]
pub struct RebalanceConfig {
    pub admin: Pubkey,
    pub rebalancing_active: bool,
}

#[account]
pub struct TokenWeight {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub target_weight: u8,
    pub current_weight: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1
    )]
    pub config: Account<'info, RebalanceConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    pool_ids: Vec<Pubkey>,
    token_mints: Vec<Pubkey>,
    weights: Vec<u8>,
    total_tokens: u64
)]
pub struct SetupInitialWeights<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, RebalanceConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: This is just used as a reference
    pub sol_mint: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, RebalanceConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Validated by Raydium program
    pub amm_program: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    pub amm_authority: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm_open_orders: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    pub market_program: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_bids: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_asks: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_event_queue: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    pub market_vault_signer: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    pub user_source_owner: Signer<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub user_token_source: UncheckedAccount<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub user_token_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapBaseIn<'info> {
    /// CHECK: Validated by Raydium program
    pub amm_program: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    pub amm_authority: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm_open_orders: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm_coin_vault: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub amm_pc_vault: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    pub market_program: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_event_queue: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_coin_vault: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub market_pc_vault: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    pub market_vault_signer: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    pub user_source_owner: Signer<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub user_token_source: AccountInfo<'info>,
    /// CHECK: Validated by Raydium program
    #[account(mut)]
    pub user_token_destination: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

fn calculate_swap_amount(
    current_balance: u64,
    current_weight: u8,
    target_weight: u8
) -> u64 {
    let weight_diff = current_weight.saturating_sub(target_weight) as u64;
    (current_balance * weight_diff) / 100
}

#[error_code]
pub enum RebalancingError {
    #[msg("Rebalancing is currently in progress")]
    RebalancingInProgress,
    #[msg("Invalid input length")]
    InvalidInputLength,
    #[msg("Weights must sum to 100")]
    InvalidWeights,
    #[msg("Swap execution failed")]
    SwapFailed,
}
