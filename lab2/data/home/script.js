//SAVE USER`S NAME
// localhost:2017
// 217.71.129.139:4798

    console.log('before');
    const container1 = document.getElementById('container');
    const container2 = document.getElementById('container2');

    if (container1) {
        // Если элементы существуют, выводим их классы
        console.log(container1.classList);
    }
    else {
        console.log('container1 не найден');
    }
    if (container2){
        console.log(container2.classList);
    } else {
        console.log('container2 не найден');
    }

async function naming(name){
    const response = await fetch("http://localhost:2017/api/session_info", {
        method: "GET",
    });
    
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    const data=await response.json();
    let header=document.getElementById("name");
    let par=document.getElementById("hash");
    console.log(data);
    if (data && data.firstName) {
        header.innerHTML = "Hello, " + data.name;
    } else {
        header.innerHTML = `Hello, ${data.name}`;
    }
    if (data && data.hash){
        par.innerHTML="Your hash:"+ data.hash.substr(0,5);
    }
}
//SUBMIT OPERATION
async function submitbtn() {
    let operation = +document.getElementById("operations").value;
    let val1 = +document.getElementById("1").value;
    let val2 = +document.getElementById("2").value;
    let result;
    if (val1!="" && val2!=""){
        const response = await fetch("http://localhost:2017/api/calculate", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                num1: val1,
                num2: val2,
                operator_id: operation,
                result: null,
            }),
        });

        if (!response.ok) {
            throw new Error(`Response status: ${response.status}`);
        }

        let label = document.getElementById("res");

        const json = await response.json();
        console.log(json);
        label.innerHTML = "Result: " + json.result;
    }
    else{
        alert("Calculator poles are empty!");
    }
    //saveOperation(`${val1} ${operation} ${val2} = ${result}`);
}
//HISTORY
async function saveOperation(operation) {
        const response = await fetch("http://localhost:2017/api/history", {
            method: "GET",
        });
        
        if (!response.ok) {
            throw new Error(`Response status: ${response.status}`);
        }

        const history = await response.json();
        const trimmed = `${json.num1} ${json.operator_id} ${json.num2}${json.result}`;
        console.log(history);
}

//LOGIN
async function login(){

    let firstName=document.getElementById("firstName").value;
    let password=document.getElementById("password").value;
    let label =document.getElementById("message");
    const response = await fetch("http://localhost:2017/api/login", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            name: firstName,
            password: password,
        }),
    });
    if (!response.ok) {
        
        label.innerHTML="Inccorrect password or login!";
        
    }
    else{
        let name=document.getElementById("firstName").value;
        alert(`Sign in was completed succesfully\nHello, ${name}`);
        
        window.location.href="/";
    }
    console.log(response);
}
document.addEventListener("DOMContentLoaded", naming);

//REGISTRATION
async function reg(){
    document.getElementById('registrationForm').addEventListener('submit', async function(event) {
        event.preventDefault();

        const firstName = document.getElementById('firstName').value;
        const password = document.getElementById('password').value;

        try {
            const response = await fetch("http://localhost:2017/api/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    name: firstName,
                    password: password
                }),
            });

            if (!response.ok) {
                throw new Error(`Server error: ${response.status}`);
            }
            window.location.href = '/';
        } catch (error) {
            console.error('Error during registration:', error);
            alert('Failed to register. Please try again.');
        }
    });
}
async function logout() {
    const response = await fetch("http://localhost:2017/api/logout", {
        method: "GET",
    });
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    else{
        window.location.href="/";
    }
}

//HISTORY

async function displayHistory() {  
    const response = await fetch("http://localhost:2017/api/history", {
        method: "GET",
    });
    
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    historyList=document.getElementById("history");
    historyList.innerHTML='';
    const history = await response.json();
    history.history.forEach(response => {
        const li = document.createElement('li');
        switch (response.operator_id){
            case 1:
                response.operator_id="+";
                break;
            case 2:
                response.operator_id="-";

                break;
            case 3:
                response.operator_id="*";
                break;
            case 4:
                response.operator_id="/";
                break;
        }
        li.textContent = `${response.num1} ${response.operator_id} ${response.num2} = ${response.result}`;
        historyList.appendChild(li);
    });
    
    console.log(history);
    
}
    /*history.forEach(operation => {
        const li = document.createElement('li');
        li.textContent = operation;
        historyList.appendChild(li);
    });*/

/*function clearHistory() {
    sessionStorage.removeItem('operationHistory');
    displayHistory();
}*/



//THEME

window.onload = function() {
    displayHistory();
    const savedTheme = sessionStorage.getItem('theme'); // Получаем сохранённую тему из sessionStorage
    const container1 = document.getElementById('container');
    const container2 = document.getElementById('container2');
    const input = document.getElementById('1');
    const input2 = document.getElementById('2');
    const select = document.getElementById('operations');
    const button = document.getElementById('theme');
    const inplog = document.getElementById('firstName');
    const pass = document.getElementById('password');

    if (savedTheme=='light-theme') {
        document.body.className = savedTheme;
        container1.className='container_light_theme';
        input.className='light-theme';
        input2.className='light-theme';
        select.className='light-theme';
        button.className='light-theme';
        inplog.className='light-theme';
        pass.className='light-theme';    
            container2.className='container2_light_theme';
    }  
    else if (savedTheme=='dark-theme'){
        document.body.className = savedTheme;
        container1.className='container_dark_theme';
        inplog.className='dark-theme';
        pass.className='dark-theme'; 
        
        input.className='dark-theme';
        input2.className='dark-theme';
        select.className='dark-theme';
        button.className='dark-theme';
        
            container2.className='container2_dark_theme';
    
    } 
    else {
        document.body.className = 'light-theme';
    }
    if (container1) {
        container1.style.display = 'block';
    }
    if (container2){
        container2.style.display = 'block';
    }
    console.log(savedTheme);
}

async function theme_changer(){
    document.getElementById('theme').value;
    const CurT = document.body.className;
    const container1 = document.getElementById('container');
    const container2 = document.getElementById('container2');
    const input = document.getElementById('1');
    const input2 = document.getElementById('2');
    const select = document.getElementById('operations');
    const button = document.getElementById('theme');
    
    if (CurT === 'light-theme') {
        document.body.className = 'dark-theme';
        sessionStorage.setItem('theme','dark-theme');
        container1.className='container_dark_theme';
        input.className='dark-theme';
        input2.className='dark-theme';
        select.className='dark-theme';
        button.className='dark-theme';
        
            container2.className='container2_dark_theme';
        
    } else {
        document.body.className = 'light-theme';
        sessionStorage.setItem('theme','light-theme');
        console.log(container1.classList);
        console.log(container2.classList);
        container1.className='container_light_theme';
        input.className='light-theme';
        input2.className='light-theme';
        select.className='light-theme';
        button.className='light-theme';
            container2.className='container2_light_theme';
    }
}
document.addEventListener("DOMContentLoaded", theme_changer);
//PROFILE IMAGE
/*async function image(){
    const response = await fetch("http://217.71.129.139:4798/api/image", {
        method: "GET",
    });
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    const data = await response.json();
    let image=document.getElementById("im");
    if (data && data.image){
        image.innerHTML=data.image;
    }
}*/
    