; ModuleID = 't_adj'
source_filename = "t_adj"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__t__adj(%Qubit* null)
  ret void
}

declare void @__quantum__qis__t__adj(%Qubit*)

attributes #0 = { "EntryPoint" "output_labeling_schema" "qir_profiles"="custom" "requiredQubits"="1" "requiredResults"="0" }
