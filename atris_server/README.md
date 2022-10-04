# Testing the server
To run the test server, run the command `cargo lambda watch`. This command expects to find a binary with the same name as the package. This is why a package with a renamed binary failed to run before

Then, in a different terminal, run `cargo lambda invoke --data-ascii '{"fullName": "[Your name]", "message": "test message"}'` to run send your request
