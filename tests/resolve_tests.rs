mod common;

use std::io::Cursor;
use std::sync::mpsc;
use glowing_fiesta::ledger::Ledger;
use crate::common::{ChannelByteReader, ChannelByteWriter, TEST_LOGS, TestLogger};
use glowing_fiesta::ledger_system::LedgerSystem;

#[test]
fn test_clean_resolve() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        dispute,1,1,\n\
        resolve,1,1,\n";
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
        1,100.0,0.0,100.0,false\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(*logs, Vec::<String>::new());
    });
}

#[test]
fn test_resolve_on_locked_account() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        dispute,1,1,\n\
        chargeback,1,1,\n\
        resolve,1,1,\n";
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

#[test]
fn test_resolve_on_a_non_existent_dispute() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        resolve,1,1,\n";
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
            vec![String::from(
                "Account (1) does not have a dispute for transaction 1"
            )]
        );
    });
}
