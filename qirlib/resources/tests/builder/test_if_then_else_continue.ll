; ModuleID = 'test_if_then_else_continue'
source_filename = "test_if_then_else_continue"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  %0 = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %0, label %then, label %else

then:                                             ; preds = %entry
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue

else:                                             ; preds = %entry
  call void @__quantum__qis__y__body(%Qubit* null)
  br label %continue

continue:                                         ; preds = %else, %then
  call void @__quantum__qis__h__body(%Qubit* null)
  ret void
}

declare void @__quantum__qis__mz__body(%Qubit*, %Result*)

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__y__body(%Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="1" }
