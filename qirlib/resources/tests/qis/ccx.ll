; ModuleID = 'ccx'
source_filename = "ccx"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__ccx__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
  ret void
}

declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)

attributes #0 = { "EntryPoint" "output_labeling_schema" "qir_profiles"="custom" "requiredQubits"="3" "requiredResults"="0" }
