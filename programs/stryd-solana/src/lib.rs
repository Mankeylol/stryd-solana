use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program;


declare_id!("ADPCeyuUkasdBcnGRDoFR4ZzmGKbsjtLW9KJwMpdX5Ce");


#[error_code]
pub enum CustomError {

    #[msg("Challenge is not pending")]
    ChallengeNotPending,

    #[msg("Challenge already has a joiner")]
    AlreadyJoined,

    #[msg("Tie not allowed")]
    TieNotAllowed,

    #[msg("Numerical overflow")]
    NumericalOverflow,

    #[msg("Insufficient funds")]
    InsufficientFunds
}

#[program]
pub mod stryd {

    use anchor_lang::system_program::Transfer;

    use super::*;

    pub fn create_challenge(ctx: Context<CreateChallenge>, challenge_id: u64, amount: u64, challenge_name: String) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        challenge.challenge_id = challenge_id;
        challenge.creator = ctx.accounts.creator.key();
        challenge.amount = amount; 
        challenge.challenge_name = challenge_name;
        challenge.joiner_distance = 0;
        challenge.creator_distance = 0;
        challenge.token_mint = Pubkey::default();
        challenge.status = ChallengeStatus::Pending;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.creator.to_account_info(),
                    to: ctx.accounts.challenge.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn join_challenge(ctx: Context<JoinChallenge>, challenge_id: u64) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
    
        require!(
            challenge.status == ChallengeStatus::Pending,
            CustomError::ChallengeNotPending
        );

        challenge.joiner = ctx.accounts.joiner.key();
        challenge.status = ChallengeStatus::Joined;

    
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.joiner.to_account_info(),
                    to: challenge.to_account_info(),
                },
            ),
            challenge.amount,
        )?;
    
        Ok(())
    }
    
    pub fn update_progress(ctx: Context<UpdateProgress>, creator_distance: u64, joiner_distance: u64, challenge_id: u64) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        challenge.creator_distance = creator_distance;
        challenge.joiner_distance = joiner_distance;
        Ok(())
    }

    pub fn resolve_challenge(ctx: Context<ResolveChallenge>) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
    
        require!(
            challenge.status == ChallengeStatus::Joined,
            CustomError::ChallengeNotPending
        );

        // Decide winner by comparing distances
        let winner_pubkey = if challenge.creator_distance > challenge.joiner_distance {
            ctx.accounts.creator.key()
        } else if challenge.joiner_distance > challenge.creator_distance {
            challenge.joiner
        } else {
            return err!(CustomError::TieNotAllowed);
        };
        challenge.winner = winner_pubkey;

        // Calculate payout (2x amount)
        let amount = challenge
            .amount
            .checked_mul(2)
            .ok_or(CustomError::NumericalOverflow)?;

        // Ensure the challenge account has enough lamports
        require!(challenge.to_account_info().lamports() >= amount, CustomError::InsufficientFunds);

        // Transfer lamports from challenge PDA to winner
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &challenge.key(),
            &winner_pubkey,
            amount,
        );

        let seeds = &[
            b"challenge",
            ctx.accounts.creator.key.as_ref(),
            &challenge.challenge_id.to_le_bytes(),
            &[ctx.bumps.challenge],
        ];

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                challenge.to_account_info(),
                ctx.accounts.winner.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[seeds],
        )?;
        challenge.status = ChallengeStatus::Resolved;

        Ok(())
    }
    
}

#[derive(Accounts)]
#[instruction(challenge_id: u64, )]
pub struct CreateChallenge<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(init,
        payer = creator,
        space = 8 + Challenge::INIT_SPACE,
        seeds = [b"challenge", creator.key().as_ref(), challenge_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub challenge: Account<'info, Challenge>,

    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
#[instruction(challenge_id: u64)]
pub struct JoinChallenge<'info> {
    #[account(mut)]
    pub joiner: Signer<'info>,


    /// CHECK: only used in PDA derivation
    pub creator: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"challenge", creator.key().as_ref(), challenge_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub challenge: Account<'info, Challenge>,

    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
#[instruction(challenge_id: u64)]
pub struct UpdateProgress<'info> {
    #[account(mut)]
    /// CHECK: only used in PDA derivation
    pub creator: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"challenge", creator.key().as_ref(), challenge_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub challenge: Account<'info, Challenge>,
}

#[derive(Accounts)]
pub struct ResolveChallenge<'info> {
    #[account(
        mut,
        seeds = [b"challenge", creator.key().as_ref(), challenge.challenge_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub challenge: Account<'info, Challenge>,

    #[account(mut)]
    pub winner: SystemAccount<'info>,

    /// CHECK: This is used only for PDA seeds
    pub creator: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}



#[account]
#[derive(InitSpace)]
pub struct Challenge {
    #[max_len(100)]
    pub challenge_name: String,
    pub challenge_id: u64,
    pub creator: Pubkey,
    pub joiner: Pubkey,
    pub creator_distance: u64,
    pub joiner_distance: u64,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub winner: Pubkey,
    pub status: ChallengeStatus,
}



#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ChallengeStatus {
    Pending,
    Joined,
    Resolved,
}
