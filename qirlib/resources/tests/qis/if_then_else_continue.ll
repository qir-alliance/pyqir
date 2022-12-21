; ModuleID = 'if_then_else_continue'
source_filename = "if_then_else_continue"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  %1 = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %1, label %then, label %else

then:                                             ; preds = %0
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue

else:                                             ; preds = %0
  call void @__quantum__qis__y__body(%Qubit* null)
  br label %continue

continue:                                         ; preds = %else, %then
  call void @__quantum__qis__h__body(%Qubit* null)
  ret void
}

declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__y__body(%Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

attributes #0 = { "EntryPoint" "output_labeling_schema" "qir_profiles"="custom" "requiredQubits"="1" "requiredResults"="1" }
attributes #1 = { "irreversible" }
