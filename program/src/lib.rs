#![cfg(feature = "program")]

use byteorder::{ByteOrder, LittleEndian};
use solana_sdk::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    info,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed,
    instruction::{Instruction}
};

use spl_token;

use std::mem;

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    // pay 100 tokens to first account

    info!("token faucet");

    // Accounts:
    //
    // receiver
    // token ()
    // faucet token account ()
    // faucet token owner ()

    // instruction_data: seed to generate faucet token account authority (programm account)

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();
    let receiver = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // this holds the faucet token account, whose authority is the program address.
    let faucet = next_account_info(accounts_iter)?;
    let faucet_authority = next_account_info(accounts_iter)?;

    let seed = instruction_data;
    // let faucet_token_account_authority = Pubkey::create_program_address(&[seed], program_id)?;

    let instruction = spl_token::instruction::transfer(
        token_program.key,
        faucet.key,
        receiver.key,
        faucet_authority.key, // generate from seed
        &[], // no multisig
        100000000)?; // decimals 8. 1 unit.

    // is this check necessary? it matters if there are multiple tokens for
    // this program to transfer from, and you want to lock the seed down.
    // if *authority_info.key != Self::authority_id(program_id, swap_info.key, nonce)? {
    //     return Err(SwapError::InvalidProgramAddress.into());
    // }

    invoke_signed(&instruction, &[
        // does order matter?
        faucet.clone(),
        receiver.clone(),
        faucet_authority.clone(),
        // why does it need token_program?
        token_program.clone(),
    ], &[&[seed]])?;


    Ok(())
}

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_sdk::clock::Epoch;

    #[test]
    fn test_sanity() {
        // let program_id = Pubkey::default();
        // let key = Pubkey::default();
        // let mut lamports = 0;
        // let mut data = vec![0; mem::size_of::<u64>()];
        // LittleEndian::write_u64(&mut data, 0);
        // let owner = Pubkey::default();
        // let account = AccountInfo::new(
        //     &key,
        //     false,
        //     true,
        //     &mut lamports,
        //     &mut data,
        //     &owner,
        //     false,
        //     Epoch::default(),
        // );

        // let mut instruction_data: Vec<u8> = vec![42, 0, 0, 0];
        // let accounts = vec![account];

        // assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 0);
        // process_instruction(&program_id, &accounts, &instruction_data).unwrap();

        // assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 42);

        // instruction_data = vec![88, 0, 0, 0];
        // process_instruction(&program_id, &accounts, &instruction_data).unwrap();

        // assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 88);
    }

    #[test]
    fn test_generate_program_address() {
        // create_program_address()
        // let i: u32 = 0;
        // let program_id = Pubkey::from_str(&"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        let program_id: Pubkey = "4JAdWitS7EDqbqHoHpo8SpXJX3kDzT5hYdMC9sWaw7ue".parse().unwrap();

        // Pubkey::find_program_address(seeds, program_id)

        let total = 100;
        let mut failures: usize = 0;
        for i in (0u32..total) {
            let mut seed = i.to_le_bytes().to_vec();

            let (address, nonce) = Pubkey::find_program_address(&[&seed], &program_id);

            println!("{} {} {}", i, address, nonce);

            // let r = Pubkey::create_program_address(&[&seed[..]], &program_id);
            // if r.is_err() {
            //     failures += 1;
            // }
        }

        println!("{}/{} program address failures", failures, total);

    }
}

// Required to support info! in tests
#[cfg(not(target_arch = "bpf"))]
solana_sdk::program_stubs!();
