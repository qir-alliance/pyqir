// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(dead_code)]
#![allow(unused_variables)]

#[repr(C)]
pub struct QirRTuple {
    private: [u8; 0],
}

pub type PauliId = i8;

use microsoft_quantum_qir_runtime_sys::runtime::{QirArray, QirRuntime, QUBIT};
use mut_static::ForceSomeRwLockWriteGuard;

use super::gates::BaseProfile;

fn get_current_gate_processor() -> ForceSomeRwLockWriteGuard<'static, BaseProfile> {
    let v = crate::gates::CURRENT_GATES.write().unwrap();
    v
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
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__h__ctl(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__h__ctl/");
    let control = get_qubit_id(ctls);
    //let mut gs = get_current_gate_processor();
    todo!("Not yet implemented.");
    //gs.h_ctl(control, get_cubit_string(qubit));
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__measure__body(
    qubits: *mut QirArray,
    registers: *mut QirArray,
) {
    log::debug!("/__quantum__qis__measure__body/");

    // get_qubit_id may return something like 94420488984834
    // which will use up all computer memory

    // let qubit = get_qubit_id(qubits);
    // let mut gs = get_current_gate_processor();
    // gs.m(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic if an unknown Pauli value is supplied.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__r__body(pauli: PauliId, theta: f64, qubit: QUBIT) {
    log::debug!("/__quantum__qis__r__body/");
    let mut gs = get_current_gate_processor();
    match pauli {
        0 => { /* identity */ }
        1 => gs.rx(theta, qubit),
        3 => gs.ry(theta, qubit),
        2 => gs.rz(theta, qubit),
        _ => panic!("Unsupported Pauli value: {}", pauli),
    }
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__r__adj(pauli: PauliId, theta: f64, qubit: QUBIT) {
    log::debug!("/__quantum__qis__r__adj/");
    //let mut gs = get_current_gate_processor();
    todo!("Not yet implemented.");
    //gs.r_adj(pauli, theta, get_cubit_string(qubit));
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__r__ctl(ctls: *mut QirArray, qubit: *mut QirRTuple) {
    log::debug!("/__quantum__qis__r__ctl/");
    todo!("Not yet implemented.");
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__r__ctladj(ctls: *mut QirArray, qubit: *mut QirRTuple) {
    log::debug!("/__quantum__qis__r__ctladj/");
    todo!("Not yet implemented.");
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
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__s__ctl(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__s__ctl/");
    todo!("Not yet implemented.");
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__s__ctladj(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__s__ctladj/");
    todo!("Not yet implemented.");
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
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__t__ctl(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__t__ctl/");
    todo!("Not yet implemented.");
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__t__ctladj(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__t__ctladj/");
    todo!("Not yet implemented.");
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
pub unsafe extern "C" fn __quantum__qis__x__ctl(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__x__ctl/");
    let control = get_qubit_id(ctls);
    let mut gs = get_current_gate_processor();
    gs.cx(control, qubit);
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
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__y__ctl(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__y__ctl/");
    todo!("Not yet implemented.");
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__z__body(qubit: QUBIT) {
    log::debug!("/__quantum__qis__z__body/");
    let mut gs = get_current_gate_processor();
    gs.y(qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__z__ctl(ctls: *mut QirArray, qubit: QUBIT) {
    log::debug!("/__quantum__qis__z__ctl/");
    let control = get_qubit_id(ctls);
    let mut gs = get_current_gate_processor();
    gs.cz(control, qubit);
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__dumpmachine__body(location: *mut u8) {
    log::debug!("/__quantum__qis__dumpmachine__body/");
    log::debug!("/__quantum__qis__h__body/");
    let mut gs = get_current_gate_processor();
    gs.dump_machine();
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
///
/// # Panics
///
/// Will panic as it is not yet implemented.
#[no_mangle]
pub unsafe extern "C" fn __quantum__qis__dumpregister__body(
    location: *mut u8,
    qubits: *mut QirArray,
) {
    log::debug!("/__quantum__qis__dumpregister__body/");
    todo!("Not yet implemented.");
}

/// # Safety
///
/// This function should not be called directly. It is intended to be
/// called by QIR applications during JIT execution.
pub unsafe fn get_qubit_id(ctls: *mut QirArray) -> QUBIT {
    #[allow(clippy::cast_ptr_alignment)]
    let ctrl_qubit_ptr = QirRuntime::quantum_rt_array_get_element_ptr_1d(ctls, 0).cast::<i64>();
    let ctrl_qubit = *ctrl_qubit_ptr;
    log::debug!("ctrl_qubit {}", ctrl_qubit);
    #[allow(clippy::cast_sign_loss)]
    let id = ctrl_qubit as QUBIT;
    id
}

/*
extern "C"
{
    // Q# Gate Set
    QIR_SHARED_API void __quantum__qis__exp__body(QirArray*, double, QirArray*); // NOLINT
    QIR_SHARED_API void __quantum__qis__exp__adj(QirArray*, double, QirArray*);  // NOLINT
    QIR_SHARED_API void __quantum__qis__exp__ctl(QirArray*, QirExpTuple*);       // NOLINT
    QIR_SHARED_API void __quantum__qis__exp__ctladj(QirArray*, QirExpTuple*);    // NOLINT
    QIR_SHARED_API void __quantum__qis__h__body(QUBIT*);                         // NOLINT
    QIR_SHARED_API void __quantum__qis__h__ctl(QirArray*, QUBIT*);               // NOLINT
    QIR_SHARED_API RESULT* __quantum__qis__measure__body(QirArray*, QirArray*);  // NOLINT
    QIR_SHARED_API void __quantum__qis__r__body(PauliId, double, QUBIT*);        // NOLINT
    QIR_SHARED_API void __quantum__qis__r__adj(PauliId, double, QUBIT*);         // NOLINT
    QIR_SHARED_API void __quantum__qis__r__ctl(QirArray*, QirRTuple*);           // NOLINT
    QIR_SHARED_API void __quantum__qis__r__ctladj(QirArray*, QirRTuple*);        // NOLINT
    QIR_SHARED_API void __quantum__qis__s__body(QUBIT*);                         // NOLINT
    QIR_SHARED_API void __quantum__qis__s__adj(QUBIT*);                          // NOLINT
    QIR_SHARED_API void __quantum__qis__s__ctl(QirArray*, QUBIT*);               // NOLINT
    QIR_SHARED_API void __quantum__qis__s__ctladj(QirArray*, QUBIT*);            // NOLINT
    QIR_SHARED_API void __quantum__qis__t__body(QUBIT*);                         // NOLINT
    QIR_SHARED_API void __quantum__qis__t__adj(QUBIT*);                          // NOLINT
    QIR_SHARED_API void __quantum__qis__t__ctl(QirArray*, QUBIT*);               // NOLINT
    QIR_SHARED_API void __quantum__qis__t__ctladj(QirArray*, QUBIT*);            // NOLINT
    QIR_SHARED_API void __quantum__qis__x__body(QUBIT*);                         // NOLINT
    QIR_SHARED_API void __quantum__qis__x__ctl(QirArray*, QUBIT*);               // NOLINT
    QIR_SHARED_API void __quantum__qis__y__body(QUBIT*);                         // NOLINT
    QIR_SHARED_API void __quantum__qis__y__ctl(QirArray*, QUBIT*);               // NOLINT
    QIR_SHARED_API void __quantum__qis__z__body(QUBIT*);                         // NOLINT
    QIR_SHARED_API void __quantum__qis__z__ctl(QirArray*, QUBIT*);               // NOLINT

    // Q# Dump:
    // Note: The param `location` must be `const void*`,
    // but it is called from .ll, where `const void*` is not supported.
    QIR_SHARED_API void __quantum__qis__dumpmachine__body(uint8_t* location);                          // NOLINT
    QIR_SHARED_API void __quantum__qis__dumpregister__body(uint8_t* location, const QirArray* qubits); // NOLINT
}
*/
