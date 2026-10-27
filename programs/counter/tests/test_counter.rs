use anchor_lang::{
    solana_program::{instruction::Instruction, pubkey::Pubkey, system_program},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn send(svm: &mut LiteSVM, ix: Instruction, payer: &Keypair, extra: &[&Keypair]) {
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra);
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let tx = Transaction::new(&signers, msg, svm.latest_blockhash());
    svm.send_transaction(tx).expect("transaction failed");
    svm.expire_blockhash();
}

fn read_count(svm: &LiteSVM, key: &Pubkey) -> u64 {
    let acc = svm.get_account(key).expect("counter account missing");
    counter::Counter::try_deserialize(&mut &acc.data[..]).unwrap().count
}

#[test]
fn counter_initializes_and_increments() {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(counter::ID, "../../target/deploy/counter.so")
        .expect("run `anchor build` first");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    let counter_kp = Keypair::new();

    let init_ix = Instruction {
        program_id: counter::ID,
        accounts: counter::accounts::Initialize {
            counter: counter_kp.pubkey(),
            user: payer.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: counter::instruction::Initialize {}.data(),
    };
    send(&mut svm, init_ix, &payer, &[&counter_kp]);
    assert_eq!(read_count(&svm, &counter_kp.pubkey()), 0);

    for expected in 1..=2u64 {
        let inc_ix = Instruction {
            program_id: counter::ID,
            accounts: counter::accounts::Increment { counter: counter_kp.pubkey() }
                .to_account_metas(None),
            data: counter::instruction::Increment {}.data(),
        };
        send(&mut svm, inc_ix, &payer, &[]);
        assert_eq!(read_count(&svm, &counter_kp.pubkey()), expected);
    }
}