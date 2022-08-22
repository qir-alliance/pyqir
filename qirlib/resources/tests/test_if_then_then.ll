; ModuleID = 'test_if_then_then'
source_filename = "test_if_then_then"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* inttoptr (i64 1 to %Result*))
  %equal = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %equal, label %then, label %else

then:                                             ; preds = %entry
  %equal1 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 1 to %Result*))
  br i1 %equal1, label %then2, label %else3

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %continue4
  ret void

then2:                                            ; preds = %then
  call void @__quantum__qis__x__body(%Qubit* null)
  br label %continue4

else3:                                            ; preds = %then
  br label %continue4

continue4:                                        ; preds = %else3, %then2
  br label %continue
}

declare void @__quantum__qis__mz__body(%Qubit*, %Result*)

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__x__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="2" }
