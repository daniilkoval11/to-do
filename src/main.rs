use std::io::{self, Write, Read};   // Этот блок обеспечивает импорт необходимых типов и макросов из библиотеки serde для работы с сериализацией и десериализацией данных.
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use chrono::{NaiveDateTime, Local};
use std::fs::{File, OpenOptions};
use serde::{Serialize, Deserialize};
use serde_json; 
use serde_repr::{Serialize_repr, Deserialize_repr};

const TASKS_FILE: &str = "tasks.json";   //Здесь объявляется константа TASKS_FILE, содержащая имя файла, в котором будут храниться задачи.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
enum Priority {  // Это определение перечисления Priority с приоритетами задач. Оно помечено атрибутами Serialize_repr и Deserialize_repr, чтобы обеспечить сериализацию и десериализацию в числовой форме.
    High,
    Medium,
    Low,
}
#[derive(Clone, Serialize, Deserialize)] 
struct Task {   // Это определение структуры Task, представляющей собой задачу. Она содержит описание, флаг завершенности, приоритет и опциональное время выполнения.
    description: String,
    completed: bool,
    priority: Priority,
    due_time: Option<NaiveDateTime>,
}

impl Priority {      
    fn color(&self) -> ColorSpec {   // Этот блок определяет метод color() для типа Priority, который возвращает спецификацию цвета на основе приоритета.
        let mut color_spec = ColorSpec::new();
        match self {
            Priority::High => color_spec.set_fg(Some(Color::Red)),
            Priority::Medium => color_spec.set_fg(Some(Color::Yellow)),
            Priority::Low => color_spec.set_fg(Some(Color::Green)),
        };
        color_spec
    }
}
struct TaskManager {    // Это определение структуры TaskManager, представляющей менеджер задач. Он содержит вектор задач.
    tasks: Vec<Task>,
}

impl TaskManager {   // В этом блоке определяются методы для структуры TaskManager, такие как new(), add_task(), complete_task(), print_tasks(), load_tasks() и save_tasks(). Эти методы выполняют операции по добавлению, завершению, выводу, загрузке и сохранению задач.
    fn new() -> Self {
        TaskManager { tasks: Vec::new() }
    }

    fn add_task(&mut self, description: String, priority: Priority, due_time: Option<NaiveDateTime>) {
        self.tasks.push(Task { description, completed: false, priority, due_time });
        self.tasks.sort_by_key(|task| task.priority);
    }

    fn complete_task(&mut self, index: usize) {
        self.tasks.get_mut(index).map(|task| task.completed = true);
    }

    fn print_tasks(&self) {
        let stdout = StandardStream::stdout(ColorChoice::Always);
        let mut stdout = stdout.lock();

        for (index, task) in self.tasks.iter().enumerate() {
            stdout.set_color(&task.priority.color()).unwrap();
            let status = if task.completed { "[x]" } else { "[ ]" };
            write!(stdout, "{} {}: {}", status, index, task.description).unwrap();

            if let Some(due_time) = task.due_time {
                let now = Local::now().naive_local();
                let (prefix, color) = if due_time > now {
                    let time_left = due_time - now;
                    (format!("Осталось {} часов", time_left.num_hours()), Color::Green)
                } else {
                    ("Просрочено".to_string(), Color::Red)
                };
                stdout.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
                writeln!(stdout, " ({})", prefix).unwrap();
                stdout.reset().unwrap();
            } else {
                writeln!(stdout).unwrap();
            }
        }
    }
}

impl Task {  // Здесь определены методы для структуры Task, такие как to_json() и from_json(), которые обеспечивают сериализацию и десериализацию задачи в формат JSON.
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn from_json(json: &str) -> Result<Task, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl TaskManager {
    fn load_tasks(&mut self) -> io::Result<()> {
        let mut file = File::open(TASKS_FILE)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let parsed_tasks: Vec<Task> = serde_json::from_str(&contents)?;
        self.tasks = parsed_tasks;

        Ok(())
    }

    fn save_tasks(&self) -> io::Result<()> {
        let tasks_json: Vec<String> = self.tasks.iter()
            .map(|task| task.to_json().unwrap())
            .collect();

        let tasks_json_str = format!("[{}]", tasks_json.join(","));

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(TASKS_FILE)?;

        file.write_all(tasks_json_str.as_bytes())?;

        Ok(())
    }
}

fn main() {  // Это функция main(), которая является точкой входа в программу. Она содержит основной цикл взаимодействия с пользователем через консоль. В зависимости от введенных команд, программа добавляет, завершает, выводит и сохраняет задачи.
    let mut task_manager = TaskManager::new(); // Создание экземпляра TaskManager, который будет управлять задачами.

    if let Err(err) = task_manager.load_tasks() { // Загрузка задач из файла при запуске программы.
        eprintln!("Ошибка при загрузке задач: {}", err);
    }

    loop {
        print!("Введите команду (add/complete/print/quit): "); // Приглашение пользователю ввести команду.
        io::stdout().flush().unwrap();  // Очистка буфера вывода.

        let mut input = String::new(); // Создание строки для хранения ввода пользователя.
        io::stdin().read_line(&mut input).unwrap();  // Считывание строки ввода.
        let command = input.trim(); // Удаление лишних пробелов из введенной строки.

        match command {
            "add" => {  // Если пользователь ввел "add", добавляем новую задачу.
                print!("Введите описание задачи: ");
                io::stdout().flush().unwrap();
                let description = read_line();

                print!("Введите приоритет задачи (high/medium/low): ");
                io::stdout().flush().unwrap();
                let priority = match read_line().as_str() {
                    "high" => Priority::High,
                    "medium" => Priority::Medium,
                    "low" => Priority::Low,
                    _ => {
                        println!("Неверный приоритет. Используется medium.");
                        Priority::Medium
                    }
                };

                print!("Введите срок выполнения (ЧЧ:ММ дд-мм-гггг) или оставьте пустым: ");
                io::stdout().flush().unwrap();
                let due_time = read_line();
                let due_time = if due_time.is_empty() {
                    None
                } else {
                    match NaiveDateTime::parse_from_str(&due_time, "%H:%M %d-%m-%Y") {
                        Ok(datetime) => Some(datetime),
                        Err(_) => {
                            println!("Неверный формат даты. Срок выполнения оставлен пустым.");
                            None
                        }
                    }
                };

                task_manager.add_task(description, priority, due_time);
                println!("Задача добавлена.");
            }
            "complete" => {  // Если пользователь ввел "complete", отмечаем задачу как завершенную.
                task_manager.print_tasks();
                print!("Введите индекс задачи для завершения: ");
                io::stdout().flush().unwrap();
                if let Ok(index) = read_line().parse::<usize>() {
                    task_manager.complete_task(index);
                    println!("Задача завершена.");
                } else {
                    println!("Неверный индекс.");
                }
            }
            "print" => {
                task_manager.print_tasks();
            }
            "quit" => {
                if let Err(err) = task_manager.save_tasks() {
                    eprintln!("Ошибка при сохранении задач: {}", err);
                }

                println!("До свидания!");
                break;
            }
            _ => {
                println!("Неизвестная команда.");
            }
        }
    }
}

fn read_line() -> String {   // Эта функция read_line() читает строку из ввода пользователя и возвращает ее, удаляя лишние пробелы.
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}


