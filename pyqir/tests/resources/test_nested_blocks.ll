; ModuleID = 'test_if'
source_filename = "test_if"

%Result = type opaque
%Qubit = type opaque

define void @main() #0 {
entry:
  %0 = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %0, label %then, label %else

then:                                             ; preds = %entry
  %1 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 1 to %Result*))
  br i1 %1, label %then1, label %else2

else:                                             ; preds = %entry
  %2 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 2 to %Result*))
  br i1 %2, label %then4, label %else5

continue:                                         ; preds = %continue6, %continue3
  ret void

then1:                                            ; preds = %then
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue3

else2:                                            ; preds = %then
  call void @__quantum__qis__y__body(%Qubit* null)
  br label %continue3

continue3:                                        ; preds = %else2, %then1
  br label %continue

then4:                                            ; preds = %else
  call void @__quantum__qis__z__body(%Qubit* null)
  br label %continue6

else5:                                            ; preds = %else
  call void @__quantum__qis__t__body(%Qubit* null)
  br label %continue6

continue6:                                        ; preds = %else5, %then4
  br label %continue
}

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__y__body(%Qubit*)

declare void @__quantum__qis__z__body(%Qubit*)

declare void @__quantum__qis__t__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="3" }
