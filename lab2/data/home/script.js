/*async function submitbtn() {
    let operation = document.getElementById("operations").value;
    let val1 = +document.getElementById("1").value;
    let val2 = +document.getElementById("2").value;

    const response = await fetch("http://localhost:3030/", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            value1: val1,
            value2: val2,
            operation: operation,
        }),
    });

    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
  
    let label = document.getElementById("res");

    const json = await response.json();
    console.log(json);

    label.innerHTML = "Result:" + json.value;
    saveOperation(`${num1} ${operation} ${num2} = ${result}`);

    function saveOperation(operation) {
        const history = JSON.parse(sessionStorage.getItem('operationHistory')) || [];
        history.push(operation);
        sessionStorage.setItem('operationHistory', JSON.stringify(history));
    }*/
        function submitbtn() {
            const num1 = parseFloat(document.getElementById('1').value);
            const num2 = parseFloat(document.getElementById('2').value);
            const operation = document.getElementById('operations').value;
            let result;
        
            switch (operation) {
                case '+':
                    result = num1 + num2;
                    break;
                case '-':
                    result = num1 - num2;
                    break;
                case '*':
                    result = num1 * num2;
                    break;
                case '/':
                    result = num1 / num2;
                    break;
                default:
                    result = 'Invalid operation';
            }
        
            document.getElementById('res').innerText = 'Result: ' + result;
        
            saveOperation(`${firstName} : ${num1} ${operation} ${num2} = ${result}`);
        }
        
        function saveOperation(operation) {
            const history = JSON.parse(sessionStorage.getItem('operationHistory')) || [];
            history.push(operation);
            
            sessionStorage.setItem('operationHistory', JSON.stringify(history));
        }
        
        const firstName = sessionStorage.getItem('firstName');
        
        if (firstName) {
            document.getElementById('welcomeMessage').innerText = `${firstName}`;
        } else {
            window.location.href = '/register/home.html';
        }

    // console.log('hello');
    // let operation = document.getElementById("operations");
    // if (operations.value=="+") {
    //     let val1 = document.getElementById("1").value;
    //     let val2 = document.getElementById("2").value;
    //     let label = document.getElementById("res");

    //     let sum = +val1 + (+val2);
    //     label.innerHTML = "Result:" + sum;
        
    // }
    // if (operations.value == "-") {
    //     let val1 = document.getElementById("1").value;
    //     let val2 = document.getElementById("2").value;
    //     let label = document.getElementById("res");

    //     let sum = +val1 - (+val2);
    //     label.innerHTML = "Result:" + sum;

    // }
    // if (operations.value == "*") {
    //     let val1 = document.getElementById("1").value;
    //     let val2 = document.getElementById("2").value;
    //     let label = document.getElementById("res");

    //     let sum =(+val1 * (+val2)) - (+val1 * (+val2)) % 0.01;
    //     label.innerHTML = "Result:" + sum;

    // }
    // if (operations.value == "/") {
    //     let val1 = document.getElementById("1").value;
    //     let val2 = document.getElementById("2").value;
    //     let label = document.getElementById("res");

    //     let sum = (+val1 / (+val2)) - (+val1 / (+val2))%0.01;

    //     label.innerHTML = "Result:" + sum;

    // }
