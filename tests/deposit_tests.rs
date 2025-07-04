mod common;

use crate::common::{ChannelByteReader, ChannelByteWriter, TEST_LOGS, TestLogger};
use glowing_fiesta::ledger::Ledger;
use glowing_fiesta::ledger_system::LedgerSystem;
use std::io::Cursor;
use std::sync::mpsc;

#[test]
fn test_clean_deposits() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        deposit,1,2,200,\n";
    let input = Cursor::new(data);
    let (tx, rx) = mpsc::channel();
    let output = ChannelByteWriter::new(tx);
    let mut output_reader = ChannelByteReader::new(rx);

    // when ...
    LedgerSystem::new(Ledger::default(), input, output).run();

    // then ...
    assert_eq!(
        output_reader.read_to_string().unwrap(),
        "client,available,held,total,locked\n\
        1,300.0,0,300.0,false\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(*logs, Vec::<String>::new());
    });
}

#[test]
fn test_deposit_without_an_amount() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        deposit,1,2,\n";
    let input = Cursor::new(data);
    let (tx, rx) = mpsc::channel();
    let output = ChannelByteWriter::new(tx);
    let mut output_reader = ChannelByteReader::new(rx);

    // when ...
    LedgerSystem::new(Ledger::default(), input, output).run();

    // then ...
    assert_eq!(
        output_reader.read_to_string().unwrap(),
        "client,available,held,total,locked\n\
        1,100.0,0,100.0,false\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(
            *logs,
            vec![String::from("Deposit transaction must have an amount")]
        );
    });
}

#[test]
fn test_deposit_on_locked() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        dispute,1,1,\n\
        chargeback,1,1,\n\
        deposit,1,2,100.0\n";
    let input = Cursor::new(data);
    let (tx, rx) = mpsc::channel();
    let output = ChannelByteWriter::new(tx);
    let mut output_reader = ChannelByteReader::new(rx);

    // when ...
    LedgerSystem::new(Ledger::default(), input, output).run();

    // then ...
    assert_eq!(
        output_reader.read_to_string().unwrap(),
        "client,available,held,total,locked\n\
        1,0.0,0.0,0.0,true\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(*logs, vec![String::from("Account (1) is locked")]);
    });
}
