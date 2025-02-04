use revm::{
    bytecode::opcode,
    context::Context,
    context_interface::TransactionType,
    database::{BenchmarkDB, FFADDRESS},
    primitives::{address, TxKind, U256},
    state::Bytecode,
    ExecuteEvm,
};

const RUNTIME_BYTECODE: &[u8] = &[
    opcode::PUSH4,
    u8::MAX,
    0,
    u8::MAX,
    u8::MAX,
    opcode::NUM2BITS,
    opcode::PUSH0,
    opcode::SSTORE,
];

fn main() -> anyhow::Result<()> {
    let auth = address!("0000000000000000000000000000000000000100");

    let bytecode = Bytecode::new_legacy(RUNTIME_BYTECODE.into());

    let mut ctx = Context::default()
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .modify_tx_chained(|tx| {
            tx.tx_type = TransactionType::Eip7702.into();
            tx.authorization_list = vec![(Some(auth), U256::from(0), 0, FFADDRESS)];
            tx.kind = TxKind::Call(auth);
        });

    let ok = ctx.exec_previous().unwrap();

    println!("{:#?}", ok.result);
    let storage = ok.state.get(&auth).unwrap().storage.clone();
    println!("{storage:#?}");

    Ok(())
}
