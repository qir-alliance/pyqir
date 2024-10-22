; ModuleID = 'if_unmeasured_result'
source_filename = "if_unmeasured_result"

define void @main() #0 {
  %1 = call i1 @__quantum__qis__read_result__body(ptr null)
  br i1 %1, label %then, label %else

then:                                             ; preds = %0
  call void @__quantum__qis__x__body(ptr null)
  br label %continue

else:                                             ; preds = %0
  call void @__quantum__qis__h__body(ptr null)
  br label %continue

continue:                                         ; preds = %else, %then
  ret void
}

declare i1 @__quantum__qis__read_result__body(ptr)

declare void @__quantum__qis__x__body(ptr)

declare void @__quantum__qis__h__body(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }
