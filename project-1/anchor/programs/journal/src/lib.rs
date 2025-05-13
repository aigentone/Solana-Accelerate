// anchor/programs/journal_program/src/lib.rs

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

declare_id!("JRNA1S7xcX6P9sS5a95hTSGmD3Yk8z123456789ABC"); // Placeholder, replace with actual

// Constants for PDA seeds
const USER_PROFILE_SEED: &[u8] = b"user_profile";
const JOURNAL_ENTRY_SEED: &[u8] = b"journal_entry";

#[program]
pub mod journal_program {
    use super::*;

    pub fn initialize_user_profile(ctx: Context<InitializeUserProfile>) -> Result<()> {
        ctx.accounts.user_profile.authority = ctx.accounts.authority.key();
        ctx.accounts.user_profile.entry_count = 0;
        ctx.accounts.user_profile.bump = ctx.bumps.user_profile;
        msg!("User profile initialized for {}", ctx.accounts.authority.key());
        Ok(())
    }

    pub fn add_journal_entry(ctx: Context<AddJournalEntry>, title: String, message: String) -> Result<()> {
        let user_profile = &mut ctx.accounts.user_profile;
        let journal_entry = &mut ctx.accounts.journal_entry;
        let authority = &ctx.accounts.authority;
        let clock = Clock::get()?;

        // Basic validation for string lengths (consider more robust checks)
        if title.len() > MAX_TITLE_LENGTH as usize {
            return err!(JournalError::TitleTooLong);
        }
        if message.len() > MAX_MESSAGE_LENGTH as usize {
            return err!(JournalError::MessageTooLong);
        }

        journal_entry.authority = authority.key();
        journal_entry.title = title;
        journal_entry.message = message;
        journal_entry.timestamp = clock.unix_timestamp;
        journal_entry.id = user_profile.entry_count;
        journal_entry.bump = ctx.bumps.journal_entry;

        user_profile.entry_count = user_profile.entry_count.checked_add(1).ok_or(JournalError::Overflow)?;
        
        msg!("Journal entry {} added for user {}", journal_entry.id, authority.key());
        Ok(())
    }

    pub fn update_journal_entry(ctx: Context<UpdateJournalEntry>, _entry_id: u64, title: String, message: String) -> Result<()> {
        let journal_entry = &mut ctx.accounts.journal_entry;
        let clock = Clock::get()?;

        if title.len() > MAX_TITLE_LENGTH as usize {
            return err!(JournalError::TitleTooLong);
        }
        if message.len() > MAX_MESSAGE_LENGTH as usize {
            return err!(JournalError::MessageTooLong);
        }
        
        journal_entry.title = title;
        journal_entry.message = message;
        journal_entry.timestamp = clock.unix_timestamp; // Update timestamp on modification

        msg!("Journal entry {} updated for user {}", journal_entry.id, ctx.accounts.authority.key());
        Ok(())
    }

    pub fn delete_journal_entry(ctx: Context<DeleteJournalEntry>, _entry_id: u64) -> Result<()> {
        // Account is closed by Anchor due to `close = authority` in `DeleteJournalEntry`
        // If we needed to adjust `user_profile.entry_count` or manage gaps, more logic would be here.
        // For simplicity, we are not compacting IDs or decrementing entry_count.
        // This means fetching all entries would require iterating up to `user_profile.entry_count`
        // and handling potential `AccountDoesNotExist` errors for deleted entries.
        msg!("Journal entry {} deleted for user {}", ctx.accounts.journal_entry.id, ctx.accounts.authority.key());
        Ok(())
    }
}

// Account Structs
const MAX_TITLE_LENGTH: u32 = 100; // 4 bytes for length + 100 bytes for string
const MAX_MESSAGE_LENGTH: u32 = 500; // 4 bytes for length + 500 bytes for string

#[account]
pub struct UserProfile {
    pub authority: Pubkey,
    pub entry_count: u64,
    pub bump: u8,
}

impl UserProfile {
    // Pubkey + u64 + u8
    pub const LEN: usize = 8 + 32 + 8 + 1;
}

#[account]
pub struct JournalEntry {
    pub authority: Pubkey,    // User who owns the entry
    pub id: u64,              // ID of the entry, specific to the user
    pub title: String,
    pub message: String,
    pub timestamp: i64,
    pub bump: u8,
}

impl JournalEntry {
    // Discriminator (8) + Pubkey (32) + u64 (8) + String (4+N) + String (4+M) + i64 (8) + u8 (1)
    // Add InitSpace trait for easier calculation if needed, or manually calculate
    pub fn space(title_len: u32, message_len: u32) -> usize {
        8 + // discriminator
        32 + // authority
        8 +  // id
        4 + title_len as usize + // title
        4 + message_len as usize + // message
        8 +  // timestamp
        1    // bump
    }
}

// Contexts for Instructions

#[derive(Accounts)]
pub struct InitializeUserProfile<'info> {
    #[account(
        init,
        payer = authority,
        space = UserProfile::LEN,
        seeds = [USER_PROFILE_SEED, authority.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, message: String)] // Used for space calculation if not using fixed max lengths
pub struct AddJournalEntry<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, authority.key().as_ref()],
        bump = user_profile.bump,
        has_one = authority, // Ensures the signer is the authority of the profile
    )]
    pub user_profile: Account<'info, UserProfile>,
    #[account(
        init,
        payer = authority,
        // Using max lengths for space calculation.
        // For dynamic sizing based on input, it's more complex and often handled by pre-calculating on client.
        // Anchor's `#[derive(InitSpace)]` helps if all fields are fixed size or have `max_len` attributes.
        // Here, we will use a fixed size based on MAX_TITLE_LENGTH and MAX_MESSAGE_LENGTH
        space = JournalEntry::space(MAX_TITLE_LENGTH, MAX_MESSAGE_LENGTH),
        seeds = [JOURNAL_ENTRY_SEED, authority.key().as_ref(), &user_profile.entry_count.to_le_bytes()],
        bump
    )]
    pub journal_entry: Account<'info, JournalEntry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(entry_id: u64, title: String, message: String)]
pub struct UpdateJournalEntry<'info> {
    #[account(
        mut,
        seeds = [JOURNAL_ENTRY_SEED, authority.key().as_ref(), &entry_id.to_le_bytes()],
        bump = journal_entry.bump,
        has_one = authority, // Ensures the signer is the authority of the entry
    )]
    pub journal_entry: Account<'info, JournalEntry>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(entry_id: u64)]
pub struct DeleteJournalEntry<'info> {
    #[account(
        mut,
        seeds = [JOURNAL_ENTRY_SEED, authority.key().as_ref(), &entry_id.to_le_bytes()],
        bump = journal_entry.bump,
        has_one = authority,
        close = authority, // Lamports from closed account are returned to the authority
    )]
    pub journal_entry: Account<'info, JournalEntry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // UserProfile is not modified here for simplicity, but could be if entry_count needs adjustment
    // #[account(
    //     mut,
    //     seeds = [USER_PROFILE_SEED, authority.key().as_ref()],
    //     bump = user_profile.bump,
    //     has_one = authority
    // )]
    // pub user_profile: Account<'info, UserProfile>,
}


// Error Enum
#[error_code]
pub enum JournalError {
    #[msg("Title is too long.")]
    TitleTooLong,
    #[msg("Message is too long.")]
    MessageTooLong,
    #[msg("Overflow occurred.")]
    Overflow,
}

Considerations for JournalEntry::space and #[derive(InitSpace)]:
The InitSpace derive macro is very helpful. To use it effectively with Strings, you'd typically add #[max_len(N)] attributes to the string fields within the struct definition.

Rust

#[account]
#[derive(InitSpace)] // Add this
pub struct JournalEntry {
    pub authority: Pubkey,
    pub id: u64,
    #[max_len(MAX_TITLE_LENGTH as usize)] // usize needed for max_len
    pub title: String,
    #[max_len(MAX_MESSAGE_LENGTH as usize)] // usize needed for max_len
    pub message: String,
    pub timestamp: i64,
    pub bump: u8,
}
Then, in AddJournalEntry, the space would be 8 + JournalEntry::INIT_SPACE.

Let's adjust JournalEntry to use InitSpace.
The MAX_TITLE_LENGTH and MAX_MESSAGE_LENGTH should represent the number of characters, not bytes including the 4-byte prefix. Anchor's #[max_len] handles the 4 + chars internally for space calculation.

Rust

// anchor/programs/journal_program/src/lib.rs

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

// IMPORTANT: Replace this with your program's actual ID after deploying/building
declare_id!("JRNA1S7xcX6P9sS5a95hTSGmD3Yk8z123456789ABC"); 

// Constants for PDA seeds
const USER_PROFILE_SEED_PREFIX: &[u8] = b"user_profile";
const JOURNAL_ENTRY_SEED_PREFIX: &[u8] = b"journal_entry";

// Constants for string lengths (characters, not including 4-byte length prefix)
const MAX_TITLE_CHARS: usize = 50; 
const MAX_MESSAGE_CHARS: usize = 280; // Like a tweet

#[program]
pub mod journal_program {
    use super::*;

    pub fn initialize_user_profile(ctx: Context<InitializeUserProfile>) -> Result<()> {
        let user_profile = &mut ctx.accounts.user_profile;
        user_profile.authority = ctx.accounts.authority.key();
        user_profile.entry_count = 0;
        user_profile.bump = ctx.bumps.user_profile;
        msg!("User profile initialized for {}", ctx.accounts.authority.key());
        Ok(())
    }

    pub fn add_journal_entry(ctx: Context<AddJournalEntry>, title: String, message: String) -> Result<()> {
        if title.chars().count() > MAX_TITLE_CHARS {
            return err!(JournalError::TitleTooLong);
        }
        if message.chars().count() > MAX_MESSAGE_CHARS {
            return err!(JournalError::MessageTooLong);
        }

        let user_profile = &mut ctx.accounts.user_profile;
        let journal_entry = &mut ctx.accounts.journal_entry;
        let authority = &ctx.accounts.authority;
        let clock = Clock::get()?;

        journal_entry.authority = authority.key();
        journal_entry.title = title;
        journal_entry.message = message;
        journal_entry.timestamp = clock.unix_timestamp;
        journal_entry.id = user_profile.entry_count; // Use current count as ID for this new entry
        journal_entry.bump = ctx.bumps.journal_entry;

        // Increment entry count for the next entry
        user_profile.entry_count = user_profile.entry_count.checked_add(1).ok_or(JournalError::Overflow)?;
        
        msg!("Journal entry {} added for user {}", journal_entry.id, authority.key());
        Ok(())
    }

    pub fn update_journal_entry(ctx: Context<UpdateJournalEntry>, _entry_id: u64, title: String, message: String) -> Result<()> {
        if title.chars().count() > MAX_TITLE_CHARS {
            return err!(JournalError::TitleTooLong);
        }
        if message.chars().count() > MAX_MESSAGE_CHARS {
            return err!(JournalError::MessageTooLong);
        }

        let journal_entry = &mut ctx.accounts.journal_entry;
        let clock = Clock::get()?;
        
        journal_entry.title = title;
        journal_entry.message = message;
        journal_entry.timestamp = clock.unix_timestamp; // Update timestamp on modification

        msg!("Journal entry {} updated for user {}", journal_entry.id, ctx.accounts.authority.key());
        Ok(())
    }

    pub fn delete_journal_entry(ctx: Context<DeleteJournalEntry>, _entry_id: u64) -> Result<()> {
        msg!("Journal entry {} with ID {} deleted for user {}", 
             ctx.accounts.journal_entry.key(), 
             ctx.accounts.journal_entry.id, 
             ctx.accounts.authority.key());
        // Account is closed by Anchor due to `close = authority` in `DeleteJournalEntry`
        // Note: This leaves a "gap" in entry_ids if user_profile.entry_count is not managed.
        // For frontend retrieval, one would iterate from 0 to user_profile.entry_count -1
        // and attempt to fetch each. If an account is not found, it's considered deleted or never existed.
        Ok(())
    }
}

// Account Structs
#[account]
#[derive(InitSpace)] // Automatically calculates space based on fields
pub struct UserProfile {
    pub authority: Pubkey,
    pub entry_count: u64, // Stores the number of entries created by this user, also used as next entry_id
    pub bump: u8,
}


#[account]
#[derive(InitSpace)]
pub struct JournalEntry {
    pub authority: Pubkey,    // User who owns the entry
    pub id: u64,              // ID of the entry, specific to the user (0, 1, 2, ...)
    #[max_len(MAX_TITLE_CHARS)]
    pub title: String,
    #[max_len(MAX_MESSAGE_CHARS)]
    pub message: String,
    pub timestamp: i64,
    pub bump: u8,
}

// Contexts for Instructions
#[derive(Accounts)]
pub struct InitializeUserProfile<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + UserProfile::INIT_SPACE, // 8 bytes for discriminator
        seeds = [USER_PROFILE_SEED_PREFIX, authority.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
// instruction macro not strictly needed here for space if using InitSpace on JournalEntry
// but can be kept for clarity or if args are used in seed paths directly in `#[account(...)]`
// #[instruction(title: String, message: String)] 
pub struct AddJournalEntry<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED_PREFIX, authority.key().as_ref()],
        bump = user_profile.bump,
        has_one = authority,
    )]
    pub user_profile: Account<'info, UserProfile>,
    #[account(
        init,
        payer = authority,
        space = 8 + JournalEntry::INIT_SPACE, // 8 bytes for discriminator
        seeds = [JOURNAL_ENTRY_SEED_PREFIX, authority.key().as_ref(), &user_profile.entry_count.to_le_bytes()],
        bump
    )]
    pub journal_entry: Account<'info, JournalEntry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(entry_id: u64)] // entry_id is used in seeds constraint
pub struct UpdateJournalEntry<'info> {
    // We need user_profile to check authority if needed, or just ensure journal_entry.authority matches signer.
    // For simplicity, keeping has_one = authority on journal_entry is sufficient.
    #[account(
        mut,
        seeds = [JOURNAL_ENTRY_SEED_PREFIX, authority.key().as_ref(), &entry_id.to_le_bytes()],
        bump = journal_entry.bump,
        has_one = authority, // This checks journal_entry.authority == authority.key()
    )]
    pub journal_entry: Account<'info, JournalEntry>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(entry_id: u64)] // entry_id is used in seeds constraint
pub struct DeleteJournalEntry<'info> {
    #[account(
        mut,
        seeds = [JOURNAL_ENTRY_SEED_PREFIX, authority.key().as_ref(), &entry_id.to_le_bytes()],
        bump = journal_entry.bump,
        has_one = authority,
        close = authority, 
    )]
    pub journal_entry: Account<'info, JournalEntry>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// Error Enum
#[error_code]
pub enum JournalError {
    #[msg("Title exceeds maximum character limit.")]
    TitleTooLong,
    #[msg("Message exceeds maximum character limit.")]
    MessageTooLong,
    #[msg("An overflow occurred.")]
    Overflow,
}