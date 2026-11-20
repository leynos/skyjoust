Feature: Bevy scenario harness

  Scenario: Steps receive the scenario metadata from the harness
    Given a scenario driven by the minimal Bevy harness
    Then the scenario metadata matches its declaration in the feature file

  Scenario: Steps share one application across ticks
    Given a scenario driven by the minimal Bevy harness
    And a toy counter incremented on every Update tick
    When the schedule advances 3 times
    Then the toy counter reads 3
    And the frame count reads 3

  Scenario: Advancing zero times leaves the application untouched
    Given a scenario driven by the minimal Bevy harness
    And a toy counter incremented on every Update tick
    When the schedule advances 0 times
    Then the toy counter reads 0
    And the frame count reads 0

  Scenario: The bare profile adds no plugins
    Given a scenario driven by the bare Bevy harness
    And a toy counter incremented on every Update tick
    When the schedule advances 2 times
    Then the toy counter reads 2
    And the application has no frame counter