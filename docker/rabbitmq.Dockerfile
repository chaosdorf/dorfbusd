FROM rabbitmq:3.8-management
RUN rabbitmq-plugins enable --offline rabbitmq_mqtt
RUN rabbitmq-plugins enable --offline rabbitmq_web_mqtt
RUN rabbitmq-plugins enable --offline rabbitmq_web_mqtt_examples 
RUN rabbitmq-plugins enable --offline rabbitmq_management