; ModuleID = 'test_if_then_else'
source_filename = "test_if_then_else"

%Result = type opaque
%Qubit = type opaque

define void @main() #0 {
entry:
  %r0 = call %Result* @__quantum__qis__m__body(%Qubit* null)
  %r1 = call %Result* @__quantum__qis__m__body(%Qubit* null)
  %one = call %Result* @__quantum__rt__result_get_one()
  %equal = call i1 @__quantum__rt__result_equal(%Result* %r0, %Result* %one)
  br i1 %equal, label %then, label %else

then:                                             ; preds = %entry
  %one1 = call %Result* @__quantum__rt__result_get_one()
  %equal2 = call i1 @__quantum__rt__result_equal(%Result* %r1, %Result* %one1)
  br i1 %equal2, label %then3, label %else4

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %continue5
  ret void

then3:                                            ; preds = %then
  br label %continue5

else4:                                            ; preds = %then
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue5

continue5:                                        ; preds = %else4, %then3
  br label %continue
}

declare %Result* @__quantum__qis__m__body(%Qubit*)

declare %Result* @__quantum__rt__result_get_one()

declare i1 @__quantum__rt__result_equal(%Result*, %Result*)

declare void @__quantum__qis__x__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" }
