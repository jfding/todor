# Design of encrypted box

Methods of how to encrypt and decrypt the box(md file).

## Cli

`todor enc <boxname>` to encrypt the box, and `todor dec <boxname>` to decrypt the box.

* `listbox` will show the encrypted box with a special flag
* `edit` will not support encrypted box with friendly tip message
* all other cmd will ask <passphrase> before operation on encrypted boxes
* will use zip crate for encryption and decryption
  * compression method is "store" for speed
  * encrypted box file will have a special extension ".mdx"
  * cmds will detect the box whether it is encrypted or not by ext name only
  * - [ ] (nice to have,) will remember the passphrase for a short time
