// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(dead_code)]
#![allow(unused_variables)]

use super::gates::BaseProfile;
use bitvec::prelude::*;
use lazy_static::lazy_static;
use mut_static::ForceSomeRwLockWriteGuard;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Mutex;

#[allow(clippy::upper_case_acronyms)]
type QUBIT = u64;
#[allow(clippy::upper_case_acronyms)]
type RESULT = u64;

lazy_static! {
    static ref RESULTS: Mutex<BitVec> = Mutex::new(bitvec![0]);
    static ref MAX_QUBIT_ID: AtomicUsize = AtomicUsize::new(0);
    static ref STATIC_RESULT_CACHE: Mutex<HashMap<RESULT, bool>> = Mutex::new(HashMap::new());
}

pub(crate) fn reset_max_qubit_id() {
    (*MAX_QUBIT_ID).store(0, Relaxed);
}

pub(crate) fn reset_static_result_cache() {
    let mut res = STATIC_RESULT_CACHE.lock().unwrap();
    res.clear();
}

/// # Panics
///
/// This function will panic if the global state cannot be locked or if the result index is too
/// large.
pub fn set_measure_stream(bits: &BitVec) {
    let mut res = RESULTS.lock().unwrap();
    let mut copy = bits.clone();
    copy.reverse();
    *res = copy;
}

fn get_current_gate_processor() -> ForceSomeRwLockWriteGuard<'static, BaseProfile> {
    let v = crate::evaluation::gates::CURRENT_GATES.write().unwrap();
    v
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__cnot__body(control: QUBIT, qubit: QUBIT) {
    log::debug!("/__quantum__qis__cnot__body/");
    let mut gs = get_current_gate_processor();
    gs.cx(control, qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__cz__body(control: QUBIT, qubit: QUBIT) {
    log::debug!("/__quantum__qis__cz__body/");
    let mut gs = get_current_gate_processor();
    gs.cz(control, qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__h__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__h__body/");
    let mut gs = get_current_gate_processor();
    gs.h(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__s__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__s__body/");
    let mut gs = get_current_gate_processor();
    gs.s(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__s__adj(qubit: QUBIT) {
    log::debug!("/__quantum__qis__s__adj/");
    let mut gs = get_current_gate_processor();
    gs.s_adj(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__t__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__t__body/");
    let mut gs = get_current_gate_processor();
    gs.t(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__t__adj(qubit: QUBIT) {
    log::debug!("/__quantum__qis__t__adj/");
    let mut gs = get_current_gate_processor();
    gs.t_adj(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__x__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__x__body/");
    let mut gs = get_current_gate_processor();
    gs.x(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__y__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__y__body/");
    let mut gs = get_current_gate_processor();
    gs.y(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__z__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__z__body/");
    let mut gs = get_current_gate_processor();
    gs.z(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__rx__body(theta: f64, qubit: QUBIT) {
    log::debug!("/__quantum__qis__rx__body/");
    let mut gs = get_current_gate_processor();
    gs.rx(theta, qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__ry__body(theta: f64, qubit: QUBIT) {
    log::debug!("/__quantum__qis__ry__body/");
    let mut gs = get_current_gate_processor();
    gs.ry(theta, qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__rz__body(theta: f64, qubit: QUBIT) {
    log::debug!("/__quantum__qis__rz__body/");
    let mut gs = get_current_gate_processor();
    gs.rz(theta, qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__reset__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__reset__body/");
    let mut gs = get_current_gate_processor();
    gs.reset(qubit);
}

/// # Panics
///
/// This function will panic if the global state cannot be locked or if the result index is too
/// large.
#[no_mangle]
pub extern "C" fn __quantum__qis__m__body(qubit: QUBIT) -> *mut c_void {
    log::debug!("/__quantum__qis__m__body/");
    let mut gs = get_current_gate_processor();
    gs.m(qubit);

    let mut res = RESULTS.lock().unwrap();

    if res.pop() == Some(true) {
        __quantum__rt__result_get_one()
    } else {
        __quantum__rt__result_get_zero()
    }
}

/// # Panics
///
/// This function will panic if the global state cannot be locked or if the result index is too
/// large.
#[no_mangle]
pub extern "C" fn __quantum__qis__mz__body(qubit: QUBIT, result: RESULT) {
    log::debug!("/__quantum__qis__mz__body/");

    let mut gs = get_current_gate_processor();
    gs.mz(qubit, result);

    let mut res = RESULTS.lock().unwrap();
    let mut cache = STATIC_RESULT_CACHE.lock().unwrap();
    if res.pop() == Some(true) {
        cache.insert(result, true);
    } else {
        cache.insert(result, false);
    }
}

/// # Panics
///
/// This function will panic if the global state cannot be locked or if the result index is too
/// large.
#[no_mangle]
pub extern "C" fn __quantum__qis__read_result__body(result: RESULT) -> bool {
    log::debug!("/__quantum__qis__read_result__body/");

    let res = RESULTS.lock().unwrap();
    let cache = STATIC_RESULT_CACHE.lock().unwrap();
    if cache.contains_key(&result) {
        cache[&result]
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn __quantum__rt__result_get_zero() -> *mut c_void {
    log::debug!("/__quantum__rt__result_get_zero/");
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn __quantum__rt__result_get_one() -> *mut c_void {
    log::debug!("/__quantum__rt__result_get_one/");
    1 as *mut c_void
}

#[no_mangle]
pub extern "C" fn __quantum__rt__result_equal(r1: *mut c_void, r2: *mut c_void) -> bool {
    log::debug!("/__quantum__rt__result_equal/");
    r1 == r2
}

#[no_mangle]
pub extern "C" fn __quantum__rt__qubit_allocate() -> QUBIT {
    log::debug!("/__quantum__rt__qubit_allocate/");
    (*MAX_QUBIT_ID).fetch_add(1, Relaxed) as QUBIT
}

#[no_mangle]
pub extern "C" fn __quantum__rt__qubit_release(qubit: QUBIT) {
    log::debug!("/__quantum__rt__qubit_release/");
    (*MAX_QUBIT_ID).fetch_sub(1, Relaxed);
}
