use anchor_lang::prelude::*;

declare_id!("94L2mJxVu6ZMmHaGsCHRQ65Kk2mea6aTnwWjSdfSsmBC"); // Replace with your new Program ID after building

#[program]
pub mod journal_pda_optimized {
    use super::*;

    // Instruction to initialize a user's journal counter
    pub fn initialize_user_journal_counter(ctx: Context<InitializeUserJournalCounter>) -> Result<()> {
        ctx.accounts.user_journal_counter.owner = *ctx.accounts.owner.key;
        ctx.accounts.user_journal_counter.last_entry_index = 0;
        // Corrected bump access:
        ctx.accounts.user_journal_counter.bump = ctx.bumps.user_journal_counter;
        msg!("User journal counter initialized for: {}", ctx.accounts.owner.key());
        Ok(())
    }

    pub fn create_journal_entry(
        ctx: Context<CreateEntry>,
        title: String,
        message: String,
    ) -> Result<()> {
        let user_journal_counter = &mut ctx.accounts.user_journal_counter;
        let current_entry_index = user_journal_counter.last_entry_index;

        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.owner = *ctx.accounts.owner.key;
        journal_entry.title = title.clone(); // Keep title for display/data
        journal_entry.message = message.clone();
        journal_entry.entry_index = current_entry_index;
        // Corrected bump access:
        journal_entry.bump = ctx.bumps.journal_entry;

        // Increment the user's entry counter for the next entry
        user_journal_counter.last_entry_index = current_entry_index.checked_add(1).ok_or_else(|| ProgramError::Custom(0))?; // Added proper error handling for overflow

        msg!("Journal Entry Created");
        msg!("Owner: {}", journal_entry.owner);
        msg!("Title: {}", title);
        msg!("Message: {}", message);
        msg!("Entry Index: {}", current_entry_index);
        Ok(())
    }

    pub fn update_journal_entry(
        ctx: Context<UpdateEntry>,
        _entry_index: u64, // entry_index is now part of seeds, so implicitly validated
        new_title: String,
        new_message: String,
    ) -> Result<()> {
        msg!("Journal Entry Updating");
        msg!("Owner: {}", ctx.accounts.owner.key());
        msg!("Entry Index: {}", ctx.accounts.journal_entry.entry_index);
        msg!("New Title: {}", new_title);
        msg!("New Message: {}", new_message);

        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.title = new_title;
        journal_entry.message = new_message;

        Ok(())
    }

    pub fn delete_journal_entry(_ctx: Context<DeleteEntry>, _entry_index: u64) -> Result<()> {
        msg!("Journal entry at index {} for owner {} deleted", _ctx.accounts.journal_entry.entry_index, _ctx.accounts.owner.key());
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct UserJournalCounter {
    pub owner: Pubkey,
    pub last_entry_index: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct JournalEntryState {
    pub owner: Pubkey,
    #[max_len(50)]
    pub title: String,
    #[max_len(280)]
    pub message: String,
    pub entry_index: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializeUserJournalCounter<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + UserJournalCounter::INIT_SPACE,
        seeds = [b"counter".as_ref(), owner.key().as_ref()],
        bump // Anchor will find and assign the canonical bump to ctx.bumps.user_journal_counter 
    )]
    pub user_journal_counter: Account<'info, UserJournalCounter>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, message: String)]
pub struct CreateEntry<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + JournalEntryState::INIT_SPACE,
        seeds = [
            owner.key().as_ref(),
            b"journal".as_ref(),
            user_journal_counter.last_entry_index.to_le_bytes().as_ref()
        ],
        bump // Anchor will find and assign the canonical bump to ctx.bumps.journal_entry
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(
        mut,
        seeds = [b"counter".as_ref(), owner.key().as_ref()],
        bump = user_journal_counter.bump
    )]
    pub user_journal_counter: Account<'info, UserJournalCounter>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(entry_index: u64, new_title: String, new_message: String)]
pub struct UpdateEntry<'info> {
    #[account(
        mut,
        seeds = [
            owner.key().as_ref(),
            b"journal".as_ref(),
            entry_index.to_le_bytes().as_ref()
        ],
        bump = journal_entry.bump,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(entry_index: u64)]
pub struct DeleteEntry<'info> {
    #[account(
        mut,
        seeds = [
            owner.key().as_ref(),
            b"journal".as_ref(),
            entry_index.to_le_bytes().as_ref()
        ],
        bump = journal_entry.bump,
        close = owner
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Added for .ok_or_else in create_journal_entry for better error handling
// You might want to define more specific errors.
// #[error_code]
// pub enum JournalError {
//     #[msg("Index overflow when creating new entry.")]
//     IndexOverflow,
// } 