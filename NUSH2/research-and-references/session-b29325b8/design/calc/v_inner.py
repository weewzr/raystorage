# JAERI inner tube heat per section, A-2. m and cp printed PDF p102, p103.
m=0.06008; cp=2871.0
T=[606,700,743,760,756]
q=[m*cp*(T[i+1]-T[i])/1000 for i in range(4)]
print([round(x,2) for x in q], round(sum(q),2), 'printed Qnaikans 25.9')
print('p in bar', round((42.42+1.033)*0.9807,2), 'p out bar', round((41.20+1.033)*0.9807,2))
print('bed density ref', 2355.2*0.55)
print('Baseline kg/kg', 3205*907.185/24/13154)
