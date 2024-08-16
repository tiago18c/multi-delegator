# Multi-Delegate SPL Token Program

This program extends the functionality of SPL token delegation by enabling multiple delegates with enhanced delegation options.

## Features

- **Multiple Delegates**: Allows token holders to assign multiple delegates for their tokens.
- **Two Delegation Types**:
  1. **Simple Delegation**: One-time delegation of tokens to a delegate.
  2. **Recurring Delegation**: Periodic delegation of tokens to a delegate over a period of time with a set amount per period.

## Instructions

The program contains 3 instructions to manage delegations:
1. `add_delegate`: Add a delegate to a token account. Can be called by both the token holder and the delegate.
2. `accept_delegate`: Accept a delegate for a token account. Can only be called by the token holder.
3. `revoke_delegate`: Revoke a delegate for a token account. Can be called by both the token holder and the delegate.

Both `add_delegate` and `accept_delegate` will setup this smart contract as the delegate for the token account when called by the token holder, ensuring that the contract can check and authorize the token delegation when the delegatee is transferring tokens.

For delegatees to exercise their delegation there are two transfer instructions:
1. `transfer`: Transfer tokens to a token account provided by the delegate. Can only be called by the delegate.
2. `transfer_recurring`: Transfer tokens to the token account configured by the delegatee in the `add_delegate` instruction. Will transfer all tokens relevant since the last transfer (or setup) according to the setup time period. Permissionless execution.


## Future Improvements and considerations

- **Timelocked Escrow Delegation**: Implement a feature to allow delegation of funds with a time-based lock, allowing for use cases that require the delegation to be locked for a specific period of time which would otherwise use pre-authorization mechanisms in traditional cards (e.g. vending machines, gas fueling, reservations, etc.)

- **Optimizations for batch transfers**: Consider storing the source token account in the delegation account, this would allow for removal of seed checks and remove the `delegate` account in the `transfer_recurring` instruction. 

- **Bankrun adoption**: Consider moving tests to bankrun to execute time-based tests without _sleeps_ 

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.