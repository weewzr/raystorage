p='engine/vv/src/harness.rs'
s=open(p,encoding='utf-8').read()
reps=[
('use nush2_plant::params::ParamSet;\n','use nush2_plant::params::ParamSet;\nuse nush2_plant::subsidy::Definition;\n'),
('''                    "-",
                    None,
                    SAME,
                    floor_note,
                ));
            }
        }''','''                    "-",
                    None,
                    SAME,
                    &format!(
                        "{floor_note}. An empty cell means that no share up to the maximum meets the target"
                    ),
                ));
            }
        }'''),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
