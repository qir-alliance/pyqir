; ModuleID = 'if_then_then'
source_filename = "if_then_then"

define void @main() #0 {
  call void @__quantum__qis__mz__body(ptr null, ptr null)
  call void @__quantum__qis__mz__body(ptr null, ptr inttoptr (i64 1 to ptr))
  %1 = call i1 @__quantum__qis__read_result__body(ptr null)
  br i1 %1, label %then, label %else

then:                                             ; preds = %0
  %2 = call i1 @__quantum__qis__read_result__body(ptr inttoptr (i64 1 to ptr))
  br i1 %2, label %then1, label %else2

else:                                             ; preds = %0
  br label %continue

continue:                                         ; preds = %else, %continue3
  ret void

then1:                                            ; preds = %then
  call void @__quantum__qis__x__body(ptr null)
  br label %continue3

else2:                                            ; preds = %then
  br label %continue3

continue3:                                        ; preds = %else2, %then1
  br label %continue
}

declare void @__quantum__qis__mz__body(ptr, ptr writeonly) #1

declare i1 @__quantum__qis__read_result__body(ptr)

declare void @__quantum__qis__x__body(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="2" }
attributes #1 = { "irreversible" }
