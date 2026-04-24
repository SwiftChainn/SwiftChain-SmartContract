import re

with open('contracts/delivery_contract/test.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'let events = env.events().all();\n    let last_event = events.last().unwrap();',
    'let events = env.events().all();\n    println!("EVENTS LEN: {}", events.len());\n    let last_event = events.last().unwrap();'
)

with open('contracts/delivery_contract/test.rs', 'w') as f:
    f.write(content)
