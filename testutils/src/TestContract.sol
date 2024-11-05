// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract TestContract {
    event BatchConfirmed(bytes32 indexed batchHeaderHash, uint32 batchId);

    function echo(bytes32 _batchHeaderHash, uint32 _batchId) public {
        emit BatchConfirmed(_batchHeaderHash, _batchId);
    }
}
