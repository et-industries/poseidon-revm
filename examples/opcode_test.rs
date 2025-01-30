use anyhow::{anyhow, bail};
use revm::{
    bytecode::opcode,
    context::Context,
    context_interface::result::{ExecutionResult, Output},
    database::CacheDB,
    database_interface::EmptyDB,
    primitives::{hex, Bytes, TxKind},
    transact_main, ExecuteCommitEvm,
};

/// Load storage from slot zero to memory
const RUNTIME_BYTECODE: &[u8] = &[
    opcode::PUSH1,
    0x02,
    opcode::PUSH1,
    0x05,
    opcode::PUSH1,
    0x03,
    opcode::MULMOD,
    opcode::PUSH0,
    opcode::SSTORE,
];

fn main() -> anyhow::Result<()> {
    let bytecode: Bytes = [RUNTIME_BYTECODE].concat().into();
    let mut ctx = Context::builder()
        .modify_tx_chained(|tx| {
            tx.kind = TxKind::Create;
            tx.data = bytecode.clone();
        })
        .with_db(CacheDB::<EmptyDB>::default());

    println!("bytecode: {}", hex::encode(bytecode));
    let ref_tx = ctx.exec_commit_previous()?;
    let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = ref_tx
    else {
        bail!("Failed to create contract: {ref_tx:#?}");
    };

    println!("Created contract at {address}");
    ctx.modify_tx(|tx| {
        tx.kind = TxKind::Call(address);
        tx.data = Default::default();
        tx.nonce += 1;
    });

    let result = transact_main(&mut ctx)?;
    println!(
        "{:#?}",
        result
            .state
            .get(&address)
            .ok_or_else(|| anyhow!("Contract not found"))?
            .storage
    );
    Ok(())
}
