// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// Reference fixture for `lancer-slither --self-test`.
/// Intentionally contains many of the patterns that the detectors look for.

contract Bad {
    address public owner;
    uint256 public balance;
    bool    public initialized;

    constructor() {
        owner = msg.sender;
    }

    function setOwner(address newOwner) public {
        owner = newOwner;
    }

    function initialize() public {
        initialized = true;
    }

    function destroy() public {
        selfdestruct(payable(msg.sender));
    }

    function withdraw(uint256 amount) public {
        payable(msg.sender).transfer(amount);
        balance = balance - amount;
    }

    function checkBalance() public view returns (bool) {
        return address(this).balance == 0;
    }

    function bad(address a) public {
        require(tx.origin == owner);
        (bool ok, ) = a.delegatecall(abi.encodeWithSignature("foo()"));
        ok;
    }

    function loopCall(address[] memory targets) public {
        for (uint256 i = 0; i < targets.length; i++) {
            targets[i].call("");
        }
    }
}
