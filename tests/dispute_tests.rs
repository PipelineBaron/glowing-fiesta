use std::io::Cursor;
use std::sync::mpsc;
use common::{ChannelByteReader, ChannelByteWriter, TestLogger, TEST_LOGS};
use glowing_fiesta::ledger::Ledger;
use glowing_fiesta::ledger_system::LedgerSystem;

mod common;

#[test]
fn test_clean_dispute() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        dispute,1,1,\n";
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
        1,0.0,100.0,100.0,false\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(*logs, Vec::<String>::new());
    });
}

#[test]
fn test_dispute_on_locked_account() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        deposit,1,2,100.0\n\
        dispute,1,1,\n\
        chargeback,1,1,\n\
        dispute,1,1,\n";
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
        1,100.0,0.0,100.0,true\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(*logs, vec![String::from("Account (1) is locked")]);
    });
}

#[test]
fn test_dispute_on_dispute_already_in_progress() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        dispute,1,1,\n\
        dispute,1,1,\n";
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
        1,0.0,100.0,100.0,false\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(
            *logs,
            vec![String::from(
                "Account (1) already has a dispute for transaction 1"
            )]
        );
    });
}

#[test]
fn test_dispute_on_withdrawal() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        withdrawal,1,2,50.0\n\
        dispute,1,2,\n";
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
        1,50.0,0,50.0,false\n"
    );
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(
            *logs,
            vec![String::from(
                "Account (1) has a dispute for withdrawal transaction 2, which is not allowed"
            )]
        );
    });
}

#[test]
fn test_dispute_on_nonexistent_transaction() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        dispute,1,2,\n";
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
            vec![String::from("Account (1) Dispute transaction 2 not found")]
        );
    });
}

#[test]
fn test_dispute_on_non_owned_transaction() {
    // given ...
    TestLogger::reset();
    let data = "type,client,tx,amount\n\
        deposit,1,1,100.0\n\
        deposit,2,2,50.0\n\
        dispute,1,2,\n";
    let input = Cursor::new(data);
    let (tx, rx) = mpsc::channel();
    let output = ChannelByteWriter::new(tx);
    let mut output_reader = ChannelByteReader::new(rx);

    // when ...
    LedgerSystem::new(Ledger::default(), input, output).run();

    // then ...
    let result = output_reader.read_to_string().unwrap();
    assert!(result.starts_with("client,available,held,total,locked\n"));
    assert!(result.contains("1,100.0,0,100.0,false\n"));
    assert!(result.contains("2,50.0,0,50.0,false\n"));
    TEST_LOGS.with_borrow(|logs| {
        assert_eq!(
            *logs,
            vec![String::from(
                "Account (1) is attempting to dispute transaction 2 owned by client 2"
            )]
        );
    });
}
