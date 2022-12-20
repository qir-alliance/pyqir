; ModuleID = 'barrier'
source_filename = "barrier"

define void @main() #0 {
  call void @__quantum__qis__barrier__body()
  ret void
}

declare void @__quantum__qis__barrier__body()

attributes #0 = { "EntryPoint" "output_labeling_schema" "qir_profiles"="custom" "requiredQubits"="0" "requiredResults"="0" }
