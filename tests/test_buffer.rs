/*
 * Copyright (c) 2020 - 2021.  Shoyo Inokuchi.
 * Please refer to github.com/shoyo/jin for more information about this project and its license.
 */

use jin::buffer::replacement::ReplacerAlgorithm;
use jin::buffer::BufferManager;
use jin::disk::DiskManager;
use std::sync::{mpsc, Arc, Barrier};
use std::thread;

mod constants;

fn setup() -> Arc<BufferManager> {
    Arc::new(BufferManager::new(
        constants::TEST_BUFFER_SIZE,
        DiskManager::new(constants::TEST_DB_FILENAME),
        ReplacerAlgorithm::Slow,
    ))
}

#[test]
fn test_create_buffer_page() {
    let manager = setup();

    // Create a page in the buffer manager.
    let frame_arc = manager.create_relation_page().unwrap();
    let frame = frame_arc.read().unwrap();

    // Assert that the created page is initialized as expected.
    assert!(frame.get_page().is_some());
    let page = frame.get_page().unwrap();
    assert_eq!(page.get_id(), constants::FIRST_RELATION_PAGE_ID);

    // Assert that new pages can't be created when the there are no open buffer frames and all
    // existing pages are pinned.
    let mut latches = Vec::new();
    for _ in 1..constants::TEST_BUFFER_SIZE {
        latches.push(manager.create_relation_page().unwrap());
    }
    assert!(manager.create_relation_page().is_err());
}

#[test]
fn test_fetch_buffer_page() {
    let manager_1 = setup();
    let manager_2 = manager_1.clone();
    let (tx, rx) = mpsc::channel();

    let handle_1 = thread::spawn(move || {
        // Assert that fetching a nonexistent page fails.
        let result = manager_1.fetch_page(constants::FIRST_RELATION_PAGE_ID);
        assert!(result.is_err());

        // Create a page and notify other threads to try to fetch the new page (should pass).
        let _ = manager_1.create_relation_page().unwrap();
        tx.send(()).unwrap();
    });

    let handle_2 = thread::spawn(move || {
        let _ = rx.recv().unwrap();
        let result = manager_2.fetch_page(constants::FIRST_RELATION_PAGE_ID);
        assert!(result.is_ok());
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();
}

#[test]
fn test_delete_buffer_page() {
    let manager_1 = setup();
    let manager_2 = manager_1.clone();
    let (tx, rx) = mpsc::channel();
    let barrier_1 = Arc::new(Barrier::new(2));
    let barrier_2 = barrier_1.clone();

    // First thread
    let handle_1 = thread::spawn(move || {
        let frame_arc = manager_1.create_relation_page().unwrap();
        let frame = frame_arc.write().unwrap();

        // Notify second thread to try to delete newly created page (should fail).
        tx.send(()).unwrap();
        barrier_1.wait();

        // Notify second thread to try again after unpinning created page (should pass).
        manager_1.unpin_w(frame);
        tx.send(()).unwrap();
    });

    // Second thread
    let handle_2 = thread::spawn(move || {
        // Receive notification from first thread to delete newly created page (should fail).
        let _ = rx.recv().unwrap();
        let first_attempt = manager_2.delete_page(constants::FIRST_RELATION_PAGE_ID);
        assert!(first_attempt.is_err());
        barrier_2.wait();

        // Receive notification from first thread to delete page again (should pass).
        let _ = rx.recv().unwrap();
        let second_attempt = manager_2.delete_page(constants::FIRST_RELATION_PAGE_ID);
        assert!(second_attempt.is_ok());
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();
}