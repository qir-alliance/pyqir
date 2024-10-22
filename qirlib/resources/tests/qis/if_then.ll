; ModuleID = 'if_then'
source_filename = "if_then"

define void @main() #0 {
  call void @__quantum__qis__mz__body(ptr null, ptr null)
  %1 = call i1 @__quantum__qis__read_result__body(ptr null)
  br i1 %1, label %then, label %else

then:                                             ; preds = %0
  call void @__quantum__qis__x__body(ptr null)
  br label %continue

else:                                             ; preds = %0
  br label %continue

continue:                                         ; preds = %else, %then
  ret void
}

declare void @__quantum__qis__mz__body(ptr, ptr writeonly) #1

declare i1 @__quantum__qis__read_result__body(ptr)

declare void @__quantum__qis__x__body(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }
attributes #1 = { "irreversible" }
