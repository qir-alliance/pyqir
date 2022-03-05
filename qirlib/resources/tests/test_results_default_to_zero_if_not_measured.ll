; ModuleID = 'test_results_default_to_zero_if_not_measured'
source_filename = "test_results_default_to_zero_if_not_measured"

%Result = type opaque
%Qubit = type opaque

define void @main() #0 {
entry:
  %zero = call %Result* @__quantum__rt__result_get_zero()
  %one = call %Result* @__quantum__rt__result_get_one()
  %equal = call i1 @__quantum__rt__result_equal(%Result* %zero, %Result* %one)
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

declare %Result* @__quantum__rt__result_get_zero()

declare %Result* @__quantum__rt__result_get_one()

declare i1 @__quantum__rt__result_equal(%Result*, %Result*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" }
