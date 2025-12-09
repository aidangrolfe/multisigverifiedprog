use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("6jLN7MDQyyEohJEWy8U5iH6oDrabje4Q4MzWqv55qWcn");

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "MultiSig Verified Progeam (TxODDSs)",
    project_url: "https://github.com/aidangrolfe/multisigverifiedprog",
    contacts: "email:aidan@txodds.com",
    policy: "https://txodds.com/security",
    preferred_languages: "en",
    source_code: "https://github.com/aidangrolfe/multisigverifiedprog",
    auditors: "Verifier pubkey: [Will be added after deployment]",
    acknowledgements: "This is a test. From TxODDs"
}

#[program]
pub mod multisig_verified {
use super::*;

    pub fn initialize(ctx: Context<Initialize>, threshold: u8) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        multisig.owners = ctx.remaining_accounts
            .iter()
            .map(|a| a.key())
            .collect::<Vec<_>>();
        require!(
            multisig.owners.len() <= 10,
            ErrorCode::TooManyOwners
        );
        require!(
            threshold as usize <= multisig.owners.len(),
            ErrorCode::InvalidThreshold
        );
        multisig.threshold = threshold;
        multisig.nonce = 0;
        Ok(())
    }

    pub fn create_transaction(
        ctx: Context<CreateTransaction>,
        instructions: Vec<TransactionInstruction>,
    ) -> Result<()> {
        let transaction = &mut ctx.accounts.transaction;
        let multisig = &ctx.accounts.multisig;
        
        transaction.multisig = multisig.key();
        transaction.instructions = instructions;
        transaction.signers = vec![false; multisig.owners.len()];
        transaction.did_execute = false;
        
        Ok(())
    }

    pub fn approve(ctx: Context<Approve>) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        let transaction = &mut ctx.accounts.transaction;
        
        let owner_index = multisig.owners
            .iter()
            .position(|a| a == ctx.accounts.owner.key)
            .ok_or(ErrorCode::InvalidOwner)?;
        
        transaction.signers[owner_index] = true;
        Ok(())
    }

    pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        let transaction = &mut ctx.accounts.transaction;
        
        require!(!transaction.did_execute, ErrorCode::AlreadyExecuted);
        
        let sig_count = transaction.signers.iter().filter(|&s| *s).count();
        require!(
            sig_count >= multisig.threshold as usize,
            ErrorCode::NotEnoughSigners
        );
        
        transaction.did_execute = true;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Multisig::MAX_SIZE
    )]
    pub multisig: Account<'info, Multisig>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTransaction<'info> {
    pub multisig: Account<'info, Multisig>,
    #[account(
        init,
        payer = proposer,
        space = 8 + Transaction::MAX_SIZE
    )]
    pub transaction: Account<'info, Transaction>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    pub multisig: Account<'info, Multisig>,
    #[account(mut, has_one = multisig)]
    pub transaction: Account<'info, Transaction>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
    pub multisig: Account<'info, Multisig>,
    #[account(mut, has_one = multisig)]
    pub transaction: Account<'info, Transaction>,
    pub executor: Signer<'info>,
}

#[account]
pub struct Multisig {
    pub owners: Vec<Pubkey>,
    pub threshold: u8,
    pub nonce: u64,
}

impl Multisig {
    pub const MAX_SIZE: usize = 32 * 10 + 1 + 8 + 100; // 10 owners max + threshold + nonce + buffer
}

#[account]
pub struct Transaction {
    pub multisig: Pubkey,
    pub instructions: Vec<TransactionInstruction>,
    pub signers: Vec<bool>,
    pub did_execute: bool,
}

impl Transaction {
    pub const MAX_SIZE: usize = 32 + 1000 + 100 + 1 + 100; // multisig + instructions + signers + executed + buffer
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransactionInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<TransactionAccount>,
    pub data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Too many owners")]
    TooManyOwners,
    #[msg("Invalid threshold")]
    InvalidThreshold,
    #[msg("Invalid owner")]
    InvalidOwner,
    #[msg("Not enough signers")]
    NotEnoughSigners,
    #[msg("Transaction already executed")]
    AlreadyExecuted,
}