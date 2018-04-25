Feature: Initialize an environment for developing a feL4 app

  Scenario: The command is issued without a conflicting directory
    Given no conflicting directory
    When the command is issued
    Then a new directory is created
    And that directory has the dependent bits for writing a feL4 app prepopulated
    And that directory contains stubs and a framework for testing the feL4 app

  Scenario: The command is issued with a conflicting directory
    Given  a conflicting directory
    When the command is run
    Then an error is returned
    And no directory is created
    And the conflicting directory is left untouched

  Scenario: The command is issued with `--bin` and without a conflicting
            directory
    Given the `--bin` flag
    And no conflicting directory
    When the command is run
    Then a new directory is created
    And that directory is populated with dependencies for building a seL4 kernel image
    And that directory contains a no-op but buildable root task
    And the build command can build a kernel image with the no-op root task without any modification
