%Result = type opaque
%Qubit = type opaque

define i64 @ENTRYPOINT__main() #0 {

entry:
  %val = add i64 0, 1
  switch i64 %val, label %default [
    i64 0, label %case0
    i64 1, label %case1
    i64 2, label %case2
  ]
case0:
  br label %measure
case1:
  ; This is the expected path for val==1
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  br label %measure
case2:
  br label %measure
default:
  br label %measure
measure:
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))

  call void @__quantum__rt__tuple_record_output(i64 1, i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
  ret i64 0
}

declare void @__quantum__qis__x__body(%Qubit*)
declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*)
declare void @__quantum__rt__tuple_record_output(i64, i8*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
attributes #1 = { "irreversible" }