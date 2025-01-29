use anyhow::bail;
use revm::{
    context_interface::result::{ExecutionResult, Output},
    database::CacheDB,
    database_interface::EmptyDB,
    primitives::{hex, Bytes, TxKind},
    transact_main, Context, ExecuteCommitEvm,
};

use alloy_sol_types::{sol, SolCall};
use ethers::abi::JsonAbi;
use ethers::contract::Lazy;

use std::include_str;

fn main() -> anyhow::Result<()> {
    // Read from the untrusted host via a Gramine-mapped file
    simulate()?;
    Ok(())
}

pub static POSEIDON_CODE: Lazy<JsonAbi> =
    Lazy::new(|| serde_json::from_str(include_str!("../out/Poseidon.sol/Poseidon.json")).unwrap());

sol! {
    function hash(bytes memory data) public view returns (bytes memory);
}

fn simulate() -> anyhow::Result<()> {
    let bytecode = Bytes::from_iter(POSEIDON_CODE.bytecode().unwrap().into_iter());
    println!("bytecode: {}", hex::encode(bytecode.clone()));

    let mut ctx = Context::builder()
        .modify_tx_chained(|tx| {
            tx.kind = TxKind::Create;
            tx.data = bytecode.clone();
        })
        .with_db(CacheDB::<EmptyDB>::default());

    let ref_tx = ctx.exec_commit_previous()?;
    let address = if let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = ref_tx
    {
        println!("contract address: {}", address);
        address
    } else {
        bail!("Failed to create contract: {ref_tx:?}");
    };

    let encoded = hashCall::new((Bytes::from_static(&[1u8; 32]),)).abi_encode();

    ctx.modify_tx(|tx| {
        tx.kind = TxKind::Call(address);
        tx.data = encoded.into();
        tx.nonce += 1;
    });

    let ref_tx = transact_main(&mut ctx).unwrap();
    let result = ref_tx.result;

    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        _ => panic!("Execution failed: {result:?}"),
    };

    let return_vals = hashCall::abi_decode_returns(&value, true)?;
    println!("result: {}", return_vals._0);

    Ok(())
}
