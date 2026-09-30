fun <TFeature, TValidated> applyToAllCloseFeaturesWithUiFlow(
    thisFeature: TFeature,
    allFeaturesOfThisKind: List<TFeature>,
    optionsToApply: TValidated,
// .. more parameters
) where TFeature : Clusterable,
        TFeature : SupportsExternalObjectCoordinates<out Options<out Options.Validated>, out Options.Validated, *>,
        TValidated : Options.Validated {
    // do something
}