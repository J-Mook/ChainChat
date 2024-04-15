import 'package:flutter/material.dart';

class InfoProvider with ChangeNotifier {

  String name = "";
  void setName(String nnn) {
    if (nnn.isNotEmpty)
      name = nnn;
    notifyListeners();
  }

  List<MessagePair> messagesList = [];
  void addmessage(bool me, String name, String mssg) {
    if(name.isNotEmpty && mssg.isNotEmpty)
      messagesList.insert(0, MessagePair(me, name, mssg));
  }

}

class MessagePair {
  bool me;
  String name;
  String msg;

  MessagePair(this.me, this.name, this.msg);
}