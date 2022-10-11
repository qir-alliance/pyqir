; ModuleID = 'test_if_then_then'
source_filename = "test_if_then_then"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* inttoptr (i64 1 to %Result*))
  %0 = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %0, label %then, label %else

then:                                             ; preds = %entry
  %1 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 1 to %Result*))
  br i1 %1, label %then1, label %else2

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %continue3
  ret void

then1:                                            ; preds = %then
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue3

else2:                                            ; preds = %then
  br label %continue3

continue3:                                        ; preds = %else2, %then1
  br label %continue
}

declare void @__quantum__qis__mz__body(%Qubit*, %Result*)

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="2" }
