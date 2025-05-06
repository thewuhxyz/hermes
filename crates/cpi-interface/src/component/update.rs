use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};


pub struct Update<'a> {
    /// Component
    pub component: &'a AccountInfo,
    /// Authority
    pub authority: &'a AccountInfo,
    /// Instruction sysvar account
    pub instruction_sysvar_account: &'a AccountInfo,
    /// Instruction
    pub component_program: &'a Pubkey,
    /// Component program
    pub instruction_data: &'a [u8],
}

impl Update<'_> {
    pub const DISCRIMINATOR: [u8; 8] = [219, 200, 88, 176, 158, 63, 253, 127];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.component.key()),
            AccountMeta::readonly_signer(self.authority.key()),
            AccountMeta::readonly(self.instruction_sysvar_account.key()),
        ];

        let instruction = Instruction {
            program_id: self.component_program,
            accounts: &account_metas,
            data: self.instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.component,
                self.authority,
                self.instruction_sysvar_account,
            ],
            signers,
        )
    }
}
