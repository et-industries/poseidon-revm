// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.8;

contract Poseidon {
    address public constant POSEIDON_ADDR =
        0x0000000000000000000000000000000000010f2C;

    function hash(bytes memory data) public view returns (bytes memory) {
        (bool success, bytes memory digest) = POSEIDON_ADDR.staticcall(data);
        require(success);
        require(digest.length == 32);
        return digest;
    }
}
