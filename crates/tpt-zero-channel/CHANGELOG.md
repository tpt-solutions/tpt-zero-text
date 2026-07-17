# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `Channel<T, N>` bounded MPSC ring with `Sender`/`Receiver` (CAS'd head/tail,
  no locking on producers).
- `TrySendError::Full` / `TryRecvError::Empty` back-pressure.
- `AcqRel`/`Acquire`/`Release` atomic ordering on slots and indices.
