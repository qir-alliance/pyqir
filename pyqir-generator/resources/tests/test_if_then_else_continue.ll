; ModuleID = 'test_if_then_else_continue'
source_filename = "test_if_then_else_continue"

%Result = type opaque
%Qubit = type opaque

define void @main() #0 {
entry:
  %r0 = call %Result* @__quantum__qis__m__body(%Qubit* null)
  %one = call %Result* @__quantum__rt__result_get_one()
  %equal = call i1 @__quantum__rt__result_equal(%Result* %r0, %Result* %one)
  br i1 %equal, label %then, label %else

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

declare %Result* @__quantum__qis__m__body(%Qubit*)

declare %Result* @__quantum__rt__result_get_one()

declare i1 @__quantum__rt__result_equal(%Result*, %Result*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__y__body(%Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" }
