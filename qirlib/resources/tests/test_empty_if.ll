; ModuleID = 'test_empty_if'
source_filename = "test_empty_if"

%Result = type opaque
%Qubit = type opaque

define void @main() #0 {
entry:
  %r0 = call %Result* @__quantum__qis__m__body(%Qubit* null)
  %one = call %Result* @__quantum__rt__result_get_one()
  %equal = call i1 @__quantum__rt__result_equal(%Result* %r0, %Result* %one)
  br i1 %equal, label %then, label %else

then:                                             ; preds = %entry
  br label %continue

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %then
  ret void
}

declare %Result* @__quantum__qis__m__body(%Qubit*)

declare %Result* @__quantum__rt__result_get_one()

declare i1 @__quantum__rt__result_equal(%Result*, %Result*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" }
