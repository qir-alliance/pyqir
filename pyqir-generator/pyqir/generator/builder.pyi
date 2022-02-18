# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

class Builder:
    """An instruction builder."""
    ...

    def call(self, name: str, *args) -> None:
        """Emits a external QIR call."""
        ...
