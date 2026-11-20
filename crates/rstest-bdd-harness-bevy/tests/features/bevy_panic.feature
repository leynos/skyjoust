Feature: Bevy scenario failure diagnostics

  Scenario: A failing step reports its scenario
    Given a scenario driven by the minimal Bevy harness
    When a step fails with "deliberate failure"