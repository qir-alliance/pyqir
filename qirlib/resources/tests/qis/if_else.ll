; ModuleID = 'if_else'
source_filename = "if_else"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  %1 = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %1, label %then, label %else

then:                                             ; preds = %0
  br label %continue

else:                                             ; preds = %0
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue

continue:                                         ; preds = %else, %then
  ret void
}

declare void @__quantum__qis__mz__body(%Qubit*, %Result*)

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="1" }
