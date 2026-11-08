p='engine/plant/src/export.rs'
s=open(p,encoding='utf-8').read()
reps=[
('''    eval: &'a SubsidyEval,
    /// The method's quantities; `None` on a row for both methods together.
    cells: Option<&'a MethodEval>,
}
''','''    eval: &'a SubsidyEval,
    /// The method's quantities; `None` on a row for both methods together.
    cells: Option<&'a MethodEval>,
    /// Source text of the row.
    source: &'a str,
}
'''),
('''        t("calculated"),
        t(SUBSIDY_SOURCE),
        t("primary"),
        t(r.equation),''','''        t("calculated"),
        t(r.source),
        t("primary"),
        t(r.equation),'''),
('''                                eval,
                                cells,
                            },
                        );''','''                                eval,
                                cells,
                                source: &source,
                            },
                        );'''),
('''    let charged = crate::charged(m, run.case, &b);
    let sp = &m.subsidy;
    for stage in Stage::ALL {''','''    let charged = crate::charged(m, run.case, &b);
    let sp = &m.subsidy;
    // The independent basis divides by the abatement against the generic
    // reference intensity (decision D2); the row says so and points to the cost
    // under the plant-built floor.
    let source = if m.carbon_params.independent() {
        format!(
            "{SUBSIDY_SOURCE}; abatement against the generic reference intensity (central), \
             with the cost and break-even shares under the plant-built floor in Final_Design_Basis.csv"
        )
    } else {
        SUBSIDY_SOURCE.to_string()
    };
    for stage in Stage::ALL {'''),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
