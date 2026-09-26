use crate::conversion::job::{ConversionPlan, ConversionPlanStep};
use crate::engines::registry::ConverterRegistry;

pub fn plan_conversion(
    from: &str,
    to: &str,
    registry: &ConverterRegistry,
) -> Option<ConversionPlan> {
    if let Some(converter) = registry.best_converter(from, to) {
        let manifest = converter.manifest();
        return Some(ConversionPlan {
            steps: vec![ConversionPlanStep {
                from: from.to_string(),
                to: to.to_string(),
                engine_id: manifest.id.clone(),
                engine_name: manifest.name.clone(),
            }],
            estimated_quality: 0.9,
            estimated_duration: "< 30s".to_string(),
        });
    }

    // Two-step route search
    let all_formats = registry.supported_outputs(from);
    for intermediate in &all_formats {
        if let (Some(c1), Some(c2)) = (
            registry.best_converter(from, intermediate),
            registry.best_converter(intermediate, to),
        ) {
            let m1 = c1.manifest();
            let m2 = c2.manifest();
            return Some(ConversionPlan {
                steps: vec![
                    ConversionPlanStep {
                        from: from.to_string(),
                        to: intermediate.clone(),
                        engine_id: m1.id.clone(),
                        engine_name: m1.name.clone(),
                    },
                    ConversionPlanStep {
                        from: intermediate.clone(),
                        to: to.to_string(),
                        engine_id: m2.id.clone(),
                        engine_name: m2.name.clone(),
                    },
                ],
                estimated_quality: 0.7,
                estimated_duration: "< 60s".to_string(),
            });
        }
    }

    None
}
