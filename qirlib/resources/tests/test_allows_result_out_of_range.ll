; ModuleID = 'test_allows_result_out_of_range'
source_filename = "test_allows_result_out_of_range"

%Result = type opaque
%Qubit = type opaque

define void @main() #0 {
entry:
  %equal = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 1 to %Result*))
  br i1 %equal, label %then, label %else

then:                                             ; preds = %entry
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue

else:                                             ; preds = %entry
  call void @__quantum__qis__h__body(%Qubit* null)
  br label %continue

continue:                                         ; preds = %else, %then
  ret void
}

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="1" }
