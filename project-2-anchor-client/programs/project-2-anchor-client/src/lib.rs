use anchor_lang::prelude::*;

declare_id!("EFN3ehsnHZcvvfQd4Fw1mTgsQb4kBBkZc5gXQrXDpsLr");

#[program]
pub mod project_2_anchor_client {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
